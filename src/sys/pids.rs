//! Process management and single-instance serialization using standard environment metadata.
//!
//! This module ensures that only a single instance of the application runs
//! concurrently by checking for an active PID file. If found, it attempts to
//! terminate the previous instance before writing its own PID atomically.
//!
//! # Note on Idioms
//! While PID files are standard in CLI tools, they have limitations (such as PID recycling).
//! For strict concurrency control in Rust, consider using OS-level file locking
//! (`flock` / `FileLock`) instead of this approach for higher reliability.

use crate::{
    AtomicWriteExt as _, Config, Environment, WallSwitchError, WallSwitchResult, get_config_path,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process, thread,
    time::Duration,
};
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Scans for existing instances and writes the current PID atomically.
///
/// # Errors
///
/// Returns a [`WallSwitchError`] if dynamic locking or directory serialization fails.
pub fn kill_other_instances(config: &Config, env: &Environment) -> WallSwitchResult<()> {
    if config.dry_run {
        if config.verbose {
            println!("[DRY-RUN] Skipping single-instance locking and termination.");
        }
        return Ok(());
    }

    // Retrieve system environment details
    let app_name = env.get_pkg_name();
    let pid_path = get_pid_path(app_name, env)?;
    let current_pid = process::id();

    // 1. Check for existing instance
    if let Some(old_pid) = read_pid_file(&pid_path)?
        && old_pid != current_pid
    {
        handle_existing_instance(&pid_path, old_pid, app_name, config)?;
    }

    // 2. Ensure parent directories exist
    ensure_parent_dir_exists(&pid_path)?;

    // 3. Write the current PID to disk atomically
    atomic_write_pid_file(&pid_path, current_pid)?;

    Ok(())
}

/// Checks if the process with `old_pid_u32` is still running and belongs to our app.
///
/// # Errors
///
/// Returns a [`WallSwitchError::IOError`] if executing process verification fails.
fn handle_existing_instance(
    pid_path: &Path,
    old_pid_u32: u32,
    app_name: &str,
    config: &Config,
) -> WallSwitchResult<()> {
    let mut sys = System::new();
    let old_pid = Pid::from_u32(old_pid_u32);

    // Refresh only the specific target PID to optimize system resource usage
    sys.refresh_processes(ProcessesToUpdate::Some(&[old_pid]), true);

    if let Some(process) = sys.process(old_pid) {
        let process_name = process.name().to_string_lossy().to_lowercase();
        let target_name_lower = app_name.to_lowercase();

        // Security check: ensure we only kill a process matching our dynamic app name
        // to avoid terminating unrelated processes that might have recycled the PID.
        if process_name.contains(&target_name_lower) {
            if config.verbose {
                println!("PID file: {}", pid_path.display());
                println!(
                    "Terminating previous instance found in PID file (PID: {})...\n",
                    old_pid_u32
                );
            }

            process.kill();

            // Give the OS scheduler time to release system resources
            thread::sleep(Duration::from_millis(200));
        } else if config.verbose {
            println!(
                "PID {} is active but process '{}' does not match '{}'. Skipping.",
                old_pid_u32, process_name, app_name
            );
        }
    }

    Ok(())
}

/// Ensures the parent directory for a given path exists.
///
/// # Errors
///
/// Returns [`WallSwitchError::IOError`] if directory creation fails.
fn ensure_parent_dir_exists(path: &Path) -> WallSwitchResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| WallSwitchError::IOError {
            path: parent.to_path_buf(),
            io_error: e,
        })?;
    }
    Ok(())
}

/// Writes the current PID to a temporary file and atomically renames it to the target path.
///
/// This atomic swap prevents partial or corrupted writes in the event of a sudden crash.
///
/// # Errors
///
/// Returns [`WallSwitchError::IOError`] if file creation, writing, or renaming fails.
fn atomic_write_pid_file(path: &Path, pid: u32) -> WallSwitchResult<()> {
    path.atomic_write(|temp_path| {
        fs::write(temp_path, pid.to_string()).map_err(|e| WallSwitchError::IOError {
            path: temp_path.to_path_buf(),
            io_error: e,
        })
    })
}

/// Reads the PID file. Returns `None` if the file is missing, empty, or corrupted.
///
/// # Errors
///
/// Returns [`WallSwitchError::IOError`] if reading the existing file fails.
fn read_pid_file(path: &Path) -> WallSwitchResult<Option<u32>> {
    if !path.exists() || !path.metadata()?.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(path).map_err(|e| WallSwitchError::IOError {
        path: path.to_path_buf(),
        io_error: e,
    })?;

    // Parse safely. If invalid (corrupted), return None to trigger self-healing.
    Ok(content.trim().parse::<u32>().ok())
}

/// Resolves the absolute path to the application PID file based on the dynamic app name.
///
/// # Errors
///
/// Returns a [`WallSwitchError`] if the standard config path resolution fails.
fn get_pid_path(app_name: &str, env: &Environment) -> WallSwitchResult<PathBuf> {
    let mut path = get_config_path(env)?;
    let pid_filename = format!("{}.pid", app_name);
    path.set_file_name(pid_filename);
    Ok(path)
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_pids {
    use super::*;

    #[test]
    fn test_read_pid_file_missing() {
        let temp_dir = std::env::temp_dir().join("wallswitch_test_missing");
        fs::create_dir_all(&temp_dir).unwrap();
        let non_existent = temp_dir.join("non_existent.pid");

        let result = read_pid_file(&non_existent);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_read_pid_file_valid() {
        let temp_dir = std::env::temp_dir().join("wallswitch_test_valid");
        fs::create_dir_all(&temp_dir).unwrap();
        let pid_file = temp_dir.join("test.pid");

        fs::write(&pid_file, "12345").unwrap();

        let result = read_pid_file(&pid_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(12345));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_read_pid_file_invalid_content() {
        let temp_dir = std::env::temp_dir().join("wallswitch_test_invalid");
        fs::create_dir_all(&temp_dir).unwrap();
        let pid_file = temp_dir.join("invalid.pid");

        fs::write(&pid_file, "not_a_number").unwrap();

        let result = read_pid_file(&pid_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_environment_pkg_name_integration() {
        let env = Environment::new();
        assert!(env.is_ok());
        let app_name = env.unwrap().get_pkg_name().to_string();
        assert!(!app_name.is_empty());
    }
}
