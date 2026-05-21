/*

src/
├── backends/             # OS-Level Side Effects (Output & Hardware)
│   ├── awww.rs           # Wayland-specific transitions using the 'awww' daemon.
│   ├── desktop.rs        # Detection and identification of the current Desktop Environment.
│   ├── detector.rs       # Discovery of active physical outputs (X11, Wayland, or DRM monitors).
│   └── wallpaper.rs      # Dispatcher logic to apply wallpapers using external tools (magick, feh).
├── cli/                  # User Interface Logic (Presentation Layer)
│   ├── args.rs           # CLI argument definitions, parsing, and shell completion generator.
│   └── list.rs           # Formatted table/JSON display and sorting of image metadata.
├── core/                 # Pure Data Models & Business Logic (Domain Layer)
│   ├── config.rs         # Merges defaults, JSON config files, and CLI overrides into a single state.
│   ├── dimension.rs      # Image geometry logic: parsing, validating, and comparing resolutions.
│   ├── fileinfo.rs       # Core data structure for image metadata (paths, hashes, sizes, mtime).
│   ├── monitors.rs       # Configuration for multi-monitor setups and output-specific settings.
│   ├── orientation.rs    # Enums and parsing for horizontal/vertical monitor layouts.
│   └── state.rs          # Manages persistent cache and history to prevent visual duplicates.
├── sys/                  # Low-Level System Integration (Input & Data Layer)
│   ├── environment.rs    # Safe access to OS environment variables ($HOME, $SESSION).
│   ├── metadata.rs       # Image metadata probing and BLAKE3 hashing.
│   ├── pids.rs           # Process management to detect and kill previous program instances.
│   └── walkdir.rs        # Recursive filesystem scanner optimized for image filtering.
├── utils/                # Generic Tools & Helpers (Shared Utilities)
│   ├── cmd.rs            # Standardized shell command execution, logging, and error handling.
│   ├── colors.rs         # ANSI styling traits for colored and formatted terminal output.
│   ├── dependencies.rs   # Pre-flight checks to verify required system binaries are installed.
│   ├── effect.rs         # Adds customizable mathematical overlays over your wallpapers.
│   ├── random.rs         # Seedless randomization and Fisher-Yates shuffling algorithms.
│   └── traits.rs         # Reusable extensions for concurrency and numeric operations.
├── app.rs                # Application Heart: Orchestrates the main program flow and run cycles.
├── error.rs              # Error Handling: Centralized custom error types and error messages.
├── lib.rs                # Library Root: Organizes modules and defines public exports.
└── main.rs               # Entry Point: Minimal bootstrap that starts the app and handles fatal exits.

*/

// ==============================================================================
// MODULE DECLARATIONS
// ==============================================================================

/// Orchestrates the core execution loops, quorum logic, and state management.
mod app;

/// Adapters for communicating with Desktop Environments, Window Managers, and Display Servers.
mod backends;

/// Adapters for user interaction, command-line arguments, and terminal output formatting.
mod cli;

/// The pure Domain of the application. Contains all business rules, entities, and validation.
mod core;

/// Global error definitions and centralized error handling logic.
mod error;

/// Adapters for interacting with the operating system (Filesystem, Processes, Environment).
mod sys;

/// Generic, domain-agnostic utilities and extension traits used across the application.
mod utils;

// ==============================================================================
// PUBLIC EXPORTS (Facade Pattern)
// ==============================================================================
// By flattening the exports here, we allow the rest of the application (like main.rs)
// to import items cleanly without needing to know the deep internal folder structure.
// Example: `use wallswitch::Config;` instead of `use wallswitch::core::config::Config;`

pub use self::{app::*, backends::*, cli::*, core::*, error::*, sys::*, utils::*};
