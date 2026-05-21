use crate::*;
use std::process;

// use rayon::prelude::*;
use std::{
    env,
    io::{self, Write},                     // For real-time terminal flushing
    sync::atomic::{AtomicUsize, Ordering}, // For thread-safe counting
    thread,
    time::Duration,
};

/// Core application logic: coordinates arguments, state, and execution cycles.
pub fn run() -> WallSwitchResult<()> {
    // 1. Parse command-line arguments as the primary source of intent
    let args = Arguments::build()?;

    // 2. Load persistent state (History and BLAKE3 hash cache) from disk
    let mut state = State::load();

    // 3. Initialize configuration by merging JSON file settings with CLI overrides
    let config = Config::new(&args)?;

    // 4. Handle listing requests
    if let Some(criteria) = args.list {
        match criteria {
            // If the user wants raw JSON state output
            SortCriteria::Processed | SortCriteria::Unprocessed | SortCriteria::Cache => {
                list_json_cache(&state, criteria)?;
            }
            // Standard table listing
            _ => {
                let mut images = gather_files(&config, &mut state)?;

                // Probe missing dimensions and validate files
                images = update_images(&images, &config, &mut state);

                list_all_images(images, criteria)?;
            }
        }
        process::exit(0);
    }

    // 5. Normal operation: Show startup info and clean up previous processes
    show_initial_msgs(&config)?;
    kill_other_instances(&config)?;

    // 6. Execute either a single switch or start the infinite loop
    if config.once {
        try_run_cycle(&config, &mut state)
    } else {
        loop {
            try_run_cycle(&config, &mut state)?;
        }
    }
}

/// Gather the files with Smart Caching and Visual Deduplication
pub fn gather_files(config: &Config, state: &mut State) -> WallSwitchResult<Vec<FileInfo>> {
    state.garbage_collect();

    let mut raw_files = Vec::new();
    for dir in &config.directories {
        raw_files.extend(get_files_from_directory(dir, config)?);
    }

    let mut needs_hash = Vec::new();
    let mut cached_files = Vec::new();

    for mut file in raw_files {
        if let Some(cache) = state.hashes.get(&file.path)
            && cache.size == file.size
            && cache.mtime == file.mtime
        {
            file.hash = cache.hash.clone();
            file.dimension = cache.dimension.clone();
            cached_files.push(file);
            continue;
        }
        needs_hash.push(file);
    }

    if !needs_hash.is_empty() {
        if config.verbose {
            println!(
                "Calculating deep BLAKE3 hashes for {} new/modified files...",
                needs_hash.len()
            );
        }
        needs_hash.update_hash()?;
    }

    for file in &needs_hash {
        state.hashes.insert(
            file.path.clone(),
            CacheEntry {
                size: file.size,
                mtime: file.mtime,
                hash: file.hash.clone(),
                dimension: file.dimension.clone(),
            },
        );
    }

    let all_files = cached_files.into_iter().chain(needs_hash);
    let mut files = Vec::new();
    let mut seen_hashes = std::collections::HashSet::new();

    for file in all_files {
        if seen_hashes.insert(file.hash.clone()) {
            files.push(file);
        } else if config.verbose {
            println!("Visual duplicate ignored: {:?}", file.path);
        }
    }

    Ok(files)
}

/// Show initial messages
pub fn show_initial_msgs(config: &Config) -> WallSwitchResult<()> {
    let env = Environment::new()?;
    let pkg_name = env.get_pkg_name();

    let pkg_desc = env!("CARGO_PKG_DESCRIPTION");
    let pkg_version = env!("CARGO_PKG_VERSION");
    let interval = config.interval;
    let info = format!("Interval between each wallpaper: {interval} seconds.");
    let author = "Claudio Fernandes de Souza Rodrigues (claudiofsrodrigues@gmail.com)";

    println!("{pkg_name} {pkg_desc}\n{info}\n{author}");
    println!("version: {pkg_version}\n");

    let depend1 = "feh (fast and light image viewer for X11/Openbox)";
    let depend2 = "awww (animated Wayland wallpaper daemon)";
    let depend3 = "swaybg (wallpaper utility for Wayland compositors)";
    let depend4 = "hyprpaper (wallpaper utility for Hyprland)";

    let dependencies = [depend1, depend2, depend3, depend4];

    println!("Dependencies:");
    dependencies.print_with_spaces(" ");
    println!();

    config.print()?;

    Ok(())
}

/// Encapsulates logic for a single wallpaper selection and application cycle.
///
/// Implements a "Strict Quorum" logic with hardware-aware concurrency.
fn try_run_cycle(config: &Config, state: &mut State) -> WallSwitchResult<()> {
    // Phase 1: Retrieve candidate files and determine optimal core counts
    let candidates = get_images(config, state)?;
    let needed = config.get_number_of_images();

    if config.verbose {
        display_files(&candidates, config);
    }

    let batch_size = candidates.get_optimal_cores();
    let mut valid_pool = accumulate_valid_pool(config, state, candidates, needed, batch_size);

    // Phase 2: Quorum Validation and Application
    if valid_pool.len() >= needed {
        let cycle_images: Vec<FileInfo> = valid_pool.drain(0..needed).collect();

        if config.verbose {
            println!(
                "Quorum satisfied: {} valid images found using {} parallel threads.",
                cycle_images.len(),
                batch_size
            );
        }

        // Output current processing queue details on terminal
        print!("{}", SliceDisplay(&cycle_images));
        println!();

        // Apply wallpapers using the selected OS-level backend (with unified on-the-fly monitor rendering)
        set_wallpaper(&cycle_images, config)?;

        // Record successful images in history and save state to disk
        for fig in &cycle_images {
            state.history.push(fig.path.clone());
        }
        state.save()?;

        if config.once {
            return Ok(());
        }

        // Wait for the configured interval before initiating the next loop cycle
        thread::sleep(Duration::from_secs(config.interval));
        return try_run_cycle(config, state);
    }

    // Phase 3: Self-Healing Trigger
    handle_history_reset_and_retry(config, state, needed, valid_pool.len())
}

/// Helper function to accumulate verified candidates until required quorum is met.
fn accumulate_valid_pool(
    config: &Config,
    state: &mut State,
    candidates: Vec<FileInfo>,
    needed: usize,
    batch_size: usize,
) -> Vec<FileInfo> {
    let mut valid_pool = Vec::new();
    let mut candidate_iter = candidates.into_iter();

    // Iterate in parallelized batches to prevent CPU starvation and balance I/O loads
    while valid_pool.len() < needed {
        let mut batch = Vec::new();

        for _ in 0..batch_size {
            if let Some(img) = candidate_iter.next() {
                batch.push(img);
            }
        }

        if batch.is_empty() {
            break;
        }

        // Perform fast-path dimension probing and validation concurrently
        let probed_batch = update_images(&batch, config, state);

        valid_pool.extend(
            probed_batch
                .into_iter()
                .filter(|f| f.is_valid == Some(true)),
        );
    }

    valid_pool
}

/// Helper function to handle self-healing when there are not enough valid images left.
fn handle_history_reset_and_retry(
    config: &Config,
    state: &mut State,
    needed: usize,
    found: usize,
) -> WallSwitchResult<()> {
    if !state.history.is_empty() {
        if config.verbose {
            println!(
                "\nQuorum failed: Needed {}, but found only {}. Resetting history for a full disk search...",
                needed, found
            );
        }
        state.history.clear();
        state.save()?;

        // Retry the cycle with a clean slate
        return try_run_cycle(config, state);
    }

    // Critical failure: No candidate files satisfied configuration rules
    Err(WallSwitchError::InsufficientNumber)
}

/// Get unique and random images filtering against history
pub fn get_images(config: &Config, state: &mut State) -> WallSwitchResult<Vec<FileInfo>> {
    let images: Vec<FileInfo> = gather_files(config, state)?;

    if images.is_empty() {
        let directories = config.directories.clone();
        return Err(WallSwitchError::NoImages { paths: directories });
    }

    // Filter out images that are already in the recent history
    let mut pool: Vec<FileInfo> = images
        .iter()
        .filter(|img| !state.history.contains(&img.path))
        .cloned()
        .collect();

    // The required number of images for ONE complete cycle
    let needed_images = config.get_number_of_images();

    // If the pool is too small to even start a cycle, reset the history immediately
    if pool.len() < needed_images {
        if config.verbose {
            println!(
                "Image pool exhausted (less than {needed_images} unseen images). Resetting history cycle."
            );
        }
        state.history.clear();
        pool = images.clone();
    }

    pool.update_number();

    if !config.sort {
        pool.shuffle();
    }

    Ok(pool)
}

/// Display found images
pub fn display_files(files: &[FileInfo], config: &Config) {
    let nfiles = files.len();
    if nfiles == 0 {
        return;
    }

    let ndigits = nfiles.to_string().len();

    if config.sort {
        println!(
            "\n{} images were found (sorted):",
            nfiles.to_string().green().bold()
        );
    } else {
        println!(
            "\n{} images were found (shuffled):",
            nfiles.to_string().green().bold()
        );
    }

    for file in files {
        println!(
            "images[{n:0ndigits$}/{t}]: {p:?}",
            n = file.number,
            p = file.path,
            t = file.total,
        );
    }
    println!();
}

/// Update FileInfo images with dimension information safely and concurrently.
pub fn update_images(files: &[FileInfo], config: &Config, state: &mut State) -> Vec<FileInfo> {
    let mut owned_files: Vec<FileInfo> = files.to_vec();

    // Identify files that lack dimension data or validation status
    let mut needs_update: Vec<&mut FileInfo> = owned_files
        .iter_mut()
        .filter(|file| file.dimension.is_none() || file.is_valid.is_none())
        .collect();

    if !needs_update.is_empty() {
        // Check how many files actually require deep probing via pure Rust image header parser
        let total_to_probe = needs_update
            .iter()
            .filter(|f| f.dimension.is_none())
            .count();

        if total_to_probe > 0 {
            // Execute multithreaded probing and validation concurrently
            probe_and_validate_parallel(&mut needs_update, config, total_to_probe);
        } else {
            // Perform low-overhead sequential validation when dimensions are already cached
            needs_update.iter_mut().for_each(|file| {
                validate_and_update(file, config);
            });
        }

        // Write newly processed dimensions back to the persistent state cache
        sync_and_save_state(&owned_files, state);
    }

    owned_files
}

// ==============================================================================
// MODULAR HELPERS (Pure and Orchestration Functions)
// ==============================================================================

/// Validates a single FileInfo against configuration constraints.
fn validate_and_update(file: &mut FileInfo, config: &Config) {
    file.is_valid = match file.validate(config) {
        Ok(()) => Some(true),
        Err(err) => {
            if config.verbose {
                log_invalid_file(file, &err);
            }
            Some(false)
        }
    };
}

/// Prints verbose details of an invalid file and its validation error to stderr.
fn log_invalid_file(file: &FileInfo, err: &impl std::fmt::Display) {
    eprintln!(
        "\n{}: {}\n-> {}",
        "Invalid file".red().bold(),
        file.path.display().to_string().yellow(),
        err
    );
}

/// Concurrently probes dimensions and validates files using thread-scoped workers.
fn probe_and_validate_parallel(
    needs_update: &mut [&mut FileInfo],
    config: &Config,
    total_to_probe: usize,
) {
    println!("Probing dimensions for {} new files...", total_to_probe);

    let width = total_to_probe.digit_width();
    let counter = AtomicUsize::new(0);
    let chunk_size = needs_update.get_chunk_size(needs_update.len());

    thread::scope(|scope| {
        for chunk in needs_update.chunks_mut(chunk_size) {
            scope.spawn(|| {
                chunk.iter_mut().for_each(|file| {
                    // 1. Probe using pure Rust image header parser if dimensions are missing
                    if file.dimension.is_none() && file.update_info(config).is_ok() {
                        let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
                        let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();

                        // Update real-time progress on stdout (overwriting same line)
                        let msg = format!(
                            "Probing image [{current:>width$}/{total_to_probe}]: {file_name}"
                        )
                        .to_line_start();
                        print!("{msg}");
                        let _ = io::stdout().flush();
                    }

                    // 2. Validate properties against configuration boundaries
                    validate_and_update(file, config);
                });
            });
        }
    });
    println!("\nProbing completed.\n");
}

/// Syncs newly discovered image dimensions back to the state cache and saves to disk.
fn sync_and_save_state(owned_files: &[FileInfo], state: &mut State) {
    let mut state_changed = false;
    for file in owned_files {
        if let Some(dim) = &file.dimension
            && let Some(entry) = state.hashes.get_mut(&file.path)
            && entry.dimension.is_none()
        {
            entry.dimension = Some(dim.clone());
            state_changed = true;
        }
    }

    if state_changed {
        let _ = state.save();
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
/// cargo test -- --show-output tests_lib
#[cfg(test)]
mod test_lib {
    use crate::*;

    #[test]
    /// `cargo test -- --show-output vec_shuffle`
    fn vec_shuffle() {
        let mut vec: Vec<u32> = (1..=100).collect();
        vec.shuffle();

        println!("vec: {vec:?}");
        assert_eq!(vec.len(), 100);
    }

    #[test]
    /// `cargo test -- --show-output random_integers_v1`
    ///
    /// <https://stackoverflow.com/questions/48218459/how-do-i-generate-a-vector-of-random-numbers-in-a-range>
    fn random_integers_v1() {
        // Example: Get a random integer value in the range 1 to 20:
        let value: u64 = get_random_integer(1, 20);

        println!("integer: {value:?}");

        // Generate a vector of 100 64-bit integer values in the range from 1 to 20,
        // allowing duplicates:

        let integers: Vec<u64> = (0..100).map(|_| get_random_integer(1, 20)).collect();

        println!("integers: {integers:?}");

        let condition_a = integers.iter().min() >= Some(&1);
        let condition_b = integers.iter().max() <= Some(&20);

        assert!(condition_a);
        assert!(condition_b);
        assert_eq!(integers.len(), 100);
    }

    #[test]
    /// `cargo test -- --show-output random_integers_v2`
    ///
    /// <https://stackoverflow.com/questions/48218459/how-do-i-generate-a-vector-of-random-numbers-in-a-range>
    fn random_integers_v2() -> WallSwitchResult<()> {
        // Example: Get a random integer value in the range 1 to 20:
        let value: u64 = get_random_integer_safe(1, 20)?;

        println!("integer: {value:?}");

        // Generate a vector of 100 64-bit integer values in the range from 1 to 20,
        // allowing duplicates:

        let integers: Vec<u64> = (0..100)
            .map(|_| get_random_integer_safe(1, 20))
            .collect::<Result<Vec<u64>, _>>()?;

        println!("integers: {integers:?}");

        let condition_a = integers.iter().min() >= Some(&1);
        let condition_b = integers.iter().max() <= Some(&20);

        assert!(condition_a);
        assert!(condition_b);
        assert_eq!(integers.len(), 100);

        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output random_integers_v3`
    fn random_integers_v3() -> WallSwitchResult<()> {
        let result = get_random_integer_safe(21, 20).map_err(|err| {
            eprintln!("{err}");
            err
        });
        assert!(result.is_err());

        let error = result.unwrap_err();
        eprintln!("error: {error:?}");

        Ok(())
    }
}
