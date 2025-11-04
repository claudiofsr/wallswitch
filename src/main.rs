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

fn main() {
    // Call the separate function that contains the main logic and can return Result
    let run_result = run();

    // Now handle the result returned by the 'run' function
    // If Error, terminate the current process with error messages.
    match run_result {
        Ok(_) => {
            process::exit(0); // Explicitly exit with success code
        }
        Err(error) => {
            eprintln!("{error}"); // Using Display prints the #[error] message
            process::exit(1); // Explicitly exit with failure code
        }
    }
}

fn run() -> WallSwitchResult<()> {
    let config = Config::new()?;
    show_initial_msgs(&config)?;
    kill_other_instances(&config)?;

    loop {
        // Count the number of valid groups of images
        let mut count: usize = 0;

        // Get random images selected from config.directories
        let images: Vec<FileInfo> = get_images(&config)?;

        if config.verbose {
            display_files(&images, &config);
        }

        // Get the number of images per cycle
        let images_per_cycle = config.get_number_of_images();
        'next: for files in images.chunks_exact(images_per_cycle) {
            if !files.sizes_are_valid(&config) {
                continue 'next;
            }

            // Update FileInfo dimension field
            let figures: Vec<FileInfo> = update_images(files, &config);

            // Print Indented file information
            print!("{}", SliceDisplay(&figures));

            for figure in &figures {
                // Go to next iteration if figure is not valid
                let dimension = figure.dimension_is_valid(&config);
                let file_name = figure.name_is_valid(&config);

                if !dimension || !file_name {
                    continue 'next;
                }
            }

            set_wallpaper(&figures, &config)?;
            sleep(Duration::from_secs(config.interval));
            count += 1;
        }

        // Make sure there are enough valid images
        if count == 0 {
            return Err(WallSwitchError::InsufficientNumber);
        }
    }
}
