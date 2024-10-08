mod config;
mod dependencies;
mod desktops;
mod dimension;
mod environment;
mod error;
mod fileinfo;
mod monitors;
mod orientation;
mod pids;
mod traits;
mod walkdir;

pub use self::{
    config::*, dependencies::*, desktops::*, dimension::*, environment::*, error::*, fileinfo::*,
    monitors::*, orientation::*, pids::*, traits::*, walkdir::*,
};

// https://crates.io/crates/cfg-if
cfg_if::cfg_if! {
    if #[cfg(feature = "args_v2")] {
        mod args_v2;
        pub use args_v2::*;
    } else {
        // default: use "clap"
        mod args_v1;
        pub use args_v1::*;
    }
}

// use rayon::prelude::*;
use std::{
    collections::HashMap,
    env,
    error::Error,
    hash::{BuildHasher, Hasher, RandomState},
    path::PathBuf,
    thread,
};

pub type MyError = Box<dyn Error + Send + Sync>;
pub type MyResult<T> = Result<T, MyError>;

/// Show initial messages
pub fn show_initial_msgs(config: &Config) -> MyResult<()> {
    let pkg_name = ENVIRON.get_pkg_name();
    let pkg_desc = env!("CARGO_PKG_DESCRIPTION");
    let pkg_version = env!("CARGO_PKG_VERSION");
    let interval = config.interval;
    let info = format!("Interval between each wallpaper: {interval} seconds.");
    let author = "Claudio Fernandes de Souza Rodrigues (claudiofsrodrigues@gmail.com)";

    println!("{pkg_name} {pkg_desc}\n{info}\n{author}");
    println!("version: {pkg_version}\n");

    let depend1 = "imagemagick (image viewing/manipulation program)";
    let depend2 = "feh (fast and light image viewer)";
    let dependencies = [depend1, depend2];

    println!("Dependencies:");
    dependencies.print_with_spaces(" ");
    println!();

    config.print()?;

    Ok(())
}

/// Get unique and random images/figures
pub fn get_images(config: &Config) -> MyResult<Vec<FileInfo>> {
    let mut images: Vec<FileInfo> = gather_files(config)?;

    if images.is_empty() {
        let directories = config.directories.clone();
        let error = WSError::NoImages(directories);
        eprintln!("{error}");
        //return Err(error.into());
        std::process::exit(1);
    }

    let nimages: usize = images.len();

    if nimages < config.monitors.len() {
        let directories: Vec<PathBuf> = images.iter().map(|f| f.path.clone()).collect();
        let error = WSError::InsufficientImages(directories, nimages);
        eprintln!("{error}");
        //return Err(error.into());
        std::process::exit(1);
    }

    images.update_number();

    if !config.sort {
        shuffle(&mut images);
    }

    Ok(images)
}

/// Gather the files in `Vec<FileInfo>`
///
/// Identical files (same hash) are disregarded
fn gather_files(config: &Config) -> MyResult<Vec<FileInfo>> {
    let mut files: Vec<FileInfo> = Vec::new();
    let mut group_by: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for dir in &config.directories {
        // Get files from directory and update hashes
        let mut infos: Vec<FileInfo> = get_files_from_directory(dir, config)?;
        infos.update_hash()?;

        for info in infos {
            let hash: String = info.hash.clone();
            let path: PathBuf = info.path.clone();

            // Insert for the first time
            if !group_by.contains_key(&hash) {
                files.push(info);
            }

            group_by.entry(hash).or_default().push(path);
        }
    }

    // Print identical file paths (if verbose mode is enabled)
    if config.verbose {
        group_by
            .values()
            .filter(|paths| paths.len() > 1)
            .for_each(|paths| {
                println!("{id}: {paths:#?}\n", id = "identical files".yellow());
            });
    }

    Ok(files)
}

/**
Shuffle the vector in place with the Fisher-Yates algorithm.

```
    use wallswitch::shuffle;

    let mut strings = vec!["abc", "foo", "bar", "baz", "mm nn", "zzz"];

    shuffle(&mut strings);

    println!("strings: {:?}", strings);

    let mut integers: Vec<u32> = (1..=20).collect();

    shuffle(&mut integers);

    println!("integers: {:?}", integers);
```

<https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle>

<https://stackoverflow.com/questions/26033976/how-do-i-create-a-vec-from-a-range-and-shuffle-it>

*/
pub fn shuffle<T>(vec: &mut [T]) {
    let n: usize = vec.len();
    for i in 0..(n - 1) {
        // Generate random index j, such that: i <= j < n
        // The remainder (`%`) after division is always less than the divisor.
        let j = (rand() as usize) % (n - i) + i;
        vec.swap(i, j);
    }
}

/// Generate a random integer value in the given range (min, max) inclusive.
pub fn get_random_integer(min: u64, max: u64) -> u64 {
    min + rand() % (max - min + 1)
}

/// Generate a random integer value in the given range (min, max) inclusive.
///
/// Return error if `min > max``
pub fn get_random_integer_v2(min: u64, max: u64) -> MyResult<u64> {
    if min > max {
        Err(WSError::MinMax(min, max).into())
    } else {
        // The remainder (`%`) after division is always less than the divisor.
        Ok(min + rand() % (max - min + 1))
    }
}

/// Generate random numbers without external dependencies
pub fn rand() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// Display found images
pub fn display_files(files: &[FileInfo], config: &Config) {
    let nfiles = files.len();
    let ndigits = nfiles.count_chars();

    if config.sort {
        println!("{nfiles} images were found (sorted):");
    } else {
        println!("{nfiles} images were found (shuffled):");
    }

    for file in files.iter() {
        println!(
            "images[{n:0ndigits$}/{t}]: {p:?}",
            n = file.number,
            p = file.path,
            t = file.total,
        );
    }
    println!();
}

/**
Update FileInfo images with dimension information

Parallelize the computation using std::thread::scope

<https://stackoverflow.com/questions/74590440/how-do-i-change-the-structure-in-the-thread>
*/
pub fn update_images(files: &[FileInfo], config: &Config) -> Vec<FileInfo> {
    let mut owned_files: Vec<FileInfo> = files.to_vec();

    thread::scope(|scope| {
        for file in &mut owned_files {
            scope.spawn(move || -> MyResult<()> {
                //let id = thread::current().id();
                //println!("identifier thread: {id:?}");
                file.update_info(config)
            });
        }
    });

    owned_files
}

/*
pub fn update_images_v2(
    files: &[FileInfo],
    config: &Config,
) -> Vec<FileInfo> {
    let mut owned_files: Vec<FileInfo> = files.to_vec();

    thread::scope(|scope| {
        let mut threads = Vec::new();

        for file in &mut owned_files {
            threads.push(scope.spawn(move || -> MyResult<()> {
                //let id = thread::current().id();
                //println!("identifier thread: {id:?}");
                file.update_info(config)
            }));
        }

        for thread in threads {
            let _result = thread.join();
        }
    });

    owned_files
}

pub fn update_images_v3(
    files: &[FileInfo],
    config: &Config,
) -> MyResult<Vec<FileInfo>> {
    files
        .iter() // Sequential computing
        //.par_iter() // Parallelize the computation using rayon
        .cloned()
        .map(|mut file| -> MyResult<FileInfo> {
            //let thread = rayon::current_thread_index();
            //println!("rayon thread id {:?}", thread);
            file.update_info(config)?;
            Ok(file)
        })
        .collect()
}
*/

#[cfg(test)]
mod test_lib {
    use crate::*;

    #[test]
    /// `cargo test -- --show-output vec_shuffle`
    fn vec_shuffle() {
        let mut vec: Vec<u32> = (1..=100).collect();
        shuffle(&mut vec);

        println!("vec: {:?}", vec);
        assert_eq!(vec.len(), 100);
    }

    #[test]
    /// `cargo test -- --show-output random_integers_v1`
    ///
    /// <https://stackoverflow.com/questions/48218459/how-do-i-generate-a-vector-of-random-numbers-in-a-range>
    fn random_integers_v1() {
        // Example: Get a random integer value in the range 1 to 20:
        let value: u64 = get_random_integer(1, 20);

        println!("integer: {:?}", value);

        // Generate a vector of 100 64-bit integer values in the range from 1 to 20,
        // allowing duplicates:

        let integers: Vec<u64> = (0..100).map(|_| get_random_integer(1, 20)).collect();

        println!("integers: {:?}", integers);

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
    fn random_integers_v2() -> MyResult<()> {
        // Example: Get a random integer value in the range 1 to 20:
        let value: u64 = get_random_integer_v2(1, 20)?;

        println!("integer: {:?}", value);

        // Generate a vector of 100 64-bit integer values in the range from 1 to 20,
        // allowing duplicates:

        let integers: Vec<u64> = (0..100)
            .map(|_| get_random_integer_v2(1, 20))
            .collect::<Result<Vec<u64>, _>>()?;

        println!("integers: {:?}", integers);

        let condition_a = integers.iter().min() >= Some(&1);
        let condition_b = integers.iter().max() <= Some(&20);

        assert!(condition_a);
        assert!(condition_b);
        assert_eq!(integers.len(), 100);

        Ok(())
    }

    #[test]
    /// `cargo test -- --show-output random_integers_v3`
    fn random_integers_v3() -> MyResult<()> {
        let result = get_random_integer_v2(21, 20).map_err(|err| {
            eprintln!("{err}");
            err
        });
        assert!(result.is_err());

        let error = result.unwrap_err();
        eprintln!("error: {error:?}");

        Ok(())
    }
}
