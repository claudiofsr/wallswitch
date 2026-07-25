use std::process;
use wallswitch::{Colors, run};

/*
cargo run
cargo run -- -m 4 -i 10
cargo run -- --once --verbose
cargo test -- --show-output
cargo doc --open
cargo b -r && cargo install --path=.
*/

/// Entry point of the application.
///
/// Manages the high-level flow and handles potential errors gracefully.
fn main() {
    let run_result = run(); // Execute core application logic.

    match run_result {
        Ok(_) => process::exit(0), // Exit successfully.
        Err(error) => {
            eprintln!("\n{} {}", "FATAL ERROR:".red().bold(), error);
            process::exit(1); // Exit with failure code.
        }
    }
}
