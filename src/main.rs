use std::process;
use wallswitch::{Colors, run};

/// Set mimalloc as the global allocator.
/// Since the unsafe operations are encapsulated inside the mimalloc library,
/// this file compiles under the strict `unsafe_code = "forbid"` rule.
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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
