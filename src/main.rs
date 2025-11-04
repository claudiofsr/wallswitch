use std::{process, thread::sleep, time::Duration};
use wallswitch::*;

/*
cargo run
cargo run -- -m 4 -i 10
cargo run --features args_v1 -- -i 4
cargo run --features args_v2 -- -i 4
cargo test -- --show-output
cargo test --features args_v2 -- --show-output
cargo fmt --all -- --check
rustfmt src/structures.rs
cargo doc --open
cargo b -r && cargo install --path=.
cargo b -r && cargo install --path=. --features args_v1
cargo b -r && cargo install --path=. --features args_v2
*/

/// Main entry point: runs application logic and handles final outcome.
fn main() {
    let run_result = run(); // Execute core application logic.

    match run_result {
        Ok(_) => process::exit(0), // Exit successfully.
        Err(error) => {
            eprintln!("{error}"); // Print error message.
            process::exit(1); // Exit with failure code.
        }
    }
}

/// Core application logic: configures, initializes, and loops wallpaper updates.
fn run() -> WallSwitchResult<()> {
    let config = Config::new()?; // Load configuration.
    show_initial_msgs(&config)?; // Display startup messages.
    kill_other_instances(&config)?; // Terminate other running instances.

    loop {
        try_run_cycle(&config)?; // Execute one wallpaper update cycle.
    }
}

/// Encapsulates logic for a single wallpaper selection and application cycle.
///
/// Finds, validates, processes, and sets images as wallpaper.
fn try_run_cycle(config: &Config) -> WallSwitchResult<()> {
    // Initialize a counter for the number of successfully processed image groups.
    // This is used to ensure at least one valid group was found and set.
    let mut count: usize = 0;

    // Get random images selected from config.directories
    let images: Vec<FileInfo> = get_images(config)?;

    if config.verbose {
        dbg!("Images obtained for this cycle.");
        display_files(&images, config);
    }

    // Get the number of images per cycle
    let images_per_cycle = config.get_number_of_images();

    'next_chunk: for files in images.chunks_exact(images_per_cycle) {
        if !files.sizes_are_valid(config) {
            dbg!("Image chunk skipped due to invalid sizes.");
            continue 'next_chunk;
        }

        // Update FileInfo dimension field
        let figures: Vec<FileInfo> = update_images(files, config);

        // Print Indented file information
        print!("{}", SliceDisplay(&figures));

        for figure in &figures {
            // Go to next iteration if figure is not valid
            let dimension = figure.dimension_is_valid(config);
            let file_name = figure.name_is_valid(config);

            if !dimension || !file_name {
                dbg!(format!(
                    "Invalid image found: {}. Skipping current chunk.",
                    figure.path.display()
                ));
                continue 'next_chunk;
            }
        }

        set_wallpaper(&figures, config)?;
        sleep(Duration::from_secs(config.interval));
        count += 1;
    }

    // Make sure there are enough valid images
    // Return Error if no valid images were set.
    if count == 0 {
        return Err(WallSwitchError::InsufficientNumber);
    }

    Ok(())
}
