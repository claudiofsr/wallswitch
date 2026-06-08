use crate::{Config, WallSwitchError, WallSwitchResult};
use std::io::Write;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Checks if a process with the specified name is currently running.
///
/// This queries the operating system's process table directly via `sysinfo`.
pub fn is_process_running(process_name: &str) -> bool {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true, // remove_dead_processes
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    sys.processes().values().any(|process| {
        process.exe().is_some_and(|exe_path| {
            exe_path
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == process_name)
        })
    })
}

/// A generic coordinator to handle background daemon processes.
///
/// This helper abstracts process state management, console visual polling feedback,
/// and startup error mapping to ensure DRY compliance across display servers.
///
/// # Errors
///
/// Returns a [`WallSwitchError`] if the daemon fails to spawn or is unresponsive after
/// the configured polling duration.
pub fn ensure_background_daemon<F, S>(
    config: &Config,
    name: &str,
    is_alive: F,
    spawn: S,
) -> WallSwitchResult<()>
where
    F: Fn() -> bool,
    S: FnOnce() -> WallSwitchResult<()>,
{
    if is_alive() {
        return Ok(());
    }

    if config.dry_run {
        println!("[DRY-RUN] {name} is down; would perform clean start.");
        return Ok(());
    }

    if config.verbose {
        println!("{name} is down. Performing clean start...");
    }

    spawn()?;

    // Poll the socket until initialized or maximum wait limit is reached.
    // Set to 5.0 seconds to allow reliable initialization on VM or heavy load.
    let max_wait = 5.0;
    let step = 0.1;
    let mut elapsed = 0.0;

    while elapsed < max_wait {
        if is_alive() {
            if config.verbose {
                println!("\n{name} successfully initialized.");
            }
            return Ok(());
        }

        if config.verbose {
            print!("\rWait to initialize {name}. Time: {elapsed:0.1}/{max_wait:0.1}");
            std::io::stdout().flush().ok();
        }

        std::thread::sleep(std::time::Duration::from_secs_f32(step));
        elapsed += step;
    }

    if config.verbose {
        println!();
    }

    Err(WallSwitchError::UnableToFind(format!(
        "{name} daemon failed to respond after initialization."
    )))
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_common_daemon {
    use super::*;

    #[test]
    fn test_is_process_running_stale_name() {
        assert!(!is_process_running("non_existent_daemon_xyz_123"));
    }
}
