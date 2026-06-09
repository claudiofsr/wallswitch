use crate::{Config, WallSwitchError, WallSwitchResult};
use std::{
    io::Write,
    process::{Command, Output},
    thread,
    time::Duration,
};
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

/// Configuration for a background daemon to ensure consistent lifecycle management.
pub struct DaemonConfig {
    /// The name of the process to check (e.g., "awww-daemon").
    pub name: &'static str,
    /// The command used to spawn the daemon.
    pub spawn_cmd: &'static str,
    /// Optional command to kill existing instances before starting (e.g., "awww-daemon").
    pub kill_cmd: Option<&'static str>,
}

/// A generic coordinator to handle background daemon processes.
///
/// This helper abstracts process state management, console visual polling feedback,
/// and startup error mapping to ensure DRY compliance across display servers.
pub struct DaemonManager;

impl DaemonManager {
    /// Ensures that a daemon is running. If not, it performs a clean start
    /// based on the provided configuration and spawn logic.
    ///
    /// # Errors
    /// Returns a [`WallSwitchError`] if the daemon fails to spawn or is unresponsive after
    /// the configured polling duration.
    pub fn ensure_running<F>(
        config: &Config,
        daemon: &DaemonConfig,
        spawn: F,
    ) -> WallSwitchResult<()>
    where
        F: FnOnce() -> WallSwitchResult<()>,
    {
        if is_process_running(daemon.name) {
            return Ok(());
        }

        if config.dry_run {
            println!(
                "[DRY-RUN] {name} is down; would perform clean start.",
                name = daemon.name
            );
            return Ok(());
        }

        if config.verbose {
            println!(
                "{name} is down. Performing clean start...",
                name = daemon.name
            );
        }

        // Clean up existing process if a kill command is specified
        if let Some(kill_name) = daemon.kill_cmd {
            let _ = Command::new("killall").arg(kill_name).output();
            // Wait briefly to allow the kernel to clean up the terminated process
            thread::sleep(Duration::from_millis(150));
        }

        spawn()?;

        // Poll the socket until initialized or maximum wait limit is reached.
        // Set to 5.0 seconds to allow reliable initialization on VM or heavy load.
        let max_wait = 5.0;
        let step = 0.1;
        let mut elapsed = 0.0;

        while elapsed < max_wait {
            if is_process_running(daemon.name) {
                if config.verbose {
                    println!("\n{name} successfully initialized.", name = daemon.name);
                }
                return Ok(());
            }

            if config.verbose {
                print!(
                    "\rWait to initialize {name}. Time: {elapsed:0.1}/{max_wait:0.1}",
                    name = daemon.name
                );
                std::io::stdout().flush().ok();
            }

            std::thread::sleep(std::time::Duration::from_secs_f32(step));
            elapsed += step;
        }

        if config.verbose {
            println!();
        }

        Err(WallSwitchError::UnableToFind(format!(
            "{name} daemon failed to respond after initialization.",
            name = daemon.name
        )))
    }
}

/// Extension trait for `std::process::Command` to unify command execution logic.
/// This replaces the manual `exec_cmd` function with a more idiomatic approach.
pub trait CommandExt {
    /// Executes the command with integrated dry-run, verbosity, and error handling.
    fn run_with_config(&mut self, config: &Config, context: &str) -> WallSwitchResult<Output>;
}

impl CommandExt for Command {
    fn run_with_config(&mut self, config: &Config, context: &str) -> WallSwitchResult<Output> {
        let output = self.output().map_err(|e| {
            eprintln!("Failed to execute command: {:?}", self.get_program());
            WallSwitchError::Io(e)
        })?;

        let program = self.get_program();
        let arguments: Vec<_> = self.get_args().collect();

        if !output.status.success() || config.verbose {
            println!("\nprogram: {program:?}");
            println!("arguments: {arguments:#?}");

            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("stdout:'{}'\n", stdout.trim());
            }
        }

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let status = output.status;

            eprintln!("{context} status: {status}");
            eprintln!("{context} stderr: {stderr}");

            return Err(WallSwitchError::CommandFailed {
                program: format!("{:?}", program),
                status: status.to_string(),
                stderr: stderr.to_string(),
            });
        }

        Ok(output)
    }
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
