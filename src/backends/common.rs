use crate::{Config, WallSwitchError, WallSwitchResult};
use std::{
    io::{Write, stdout},
    process::{Command, Output, Stdio},
    thread::sleep,
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Configuration for a background daemon to ensure consistent lifecycle management.
pub struct DaemonConfig {
    /// The name of the process to check (e.g., "awww-daemon").
    pub cmd_name: &'static str,
    /// Optional hook to run before spawning the daemon (e.g., cleaning sockets).
    pub pre_spawn_hook: Option<fn() -> WallSwitchResult<()>>,
}

/// A generic coordinator to handle background daemon processes.
///
/// This helper abstracts process state management, including automatic termination
/// of existing instances, pre-spawn hooks, and polling feedback to ensure
/// the daemon is ready before the application continues.
pub struct DaemonManager;

impl DaemonManager {
    /// Ensures that a daemon is running.
    ///
    /// If the process is not running, this method will:
    /// 1. Terminate any existing processes with the same name.
    /// 2. Execute any optional `pre_spawn_hook`.
    /// 3. Spawn the daemon.
    /// 4. Poll the system until the process is active or a timeout is reached.
    ///
    /// To optimize resource usage, a single [`System`] state is allocated here and
    /// shared across the initial query and the subsequent waiting loop.
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchError`] if:
    /// - The pre-spawn hook fails.
    /// - The daemon process fails to spawn.
    /// - The process does not initialize within the timeout window.
    pub fn ensure_running(config: &Config, daemon: &DaemonConfig) -> WallSwitchResult<()> {
        // Allocate a single System instance to share across the checks
        let mut sys = System::new();

        if is_process_running_shared(&mut sys, daemon.cmd_name) {
            return Ok(());
        }

        if config.dry_run {
            println!(
                "[DRY-RUN] {name} is down; would perform clean start.",
                name = daemon.cmd_name
            );
            let cmd = Command::new(daemon.cmd_name);
            println!("[DRY-RUN] Would execute: {:?}", cmd);
            return Ok(());
        }

        if config.verbose {
            println!(
                "{name} is down. Performing clean start...",
                name = daemon.cmd_name
            );
        }

        // 1. Terminate existing processes by name.
        terminate_processes_by_name(daemon.cmd_name);

        // 2. Execute custom initialization logic.
        if let Some(hook) = daemon.pre_spawn_hook {
            hook()?;
        }

        // 3. Spawning the daemon process.
        let mut cmd = Command::new(daemon.cmd_name);
        cmd.stdout(Stdio::null()).stderr(Stdio::null());

        let name = daemon.cmd_name.to_string();
        cmd.spawn()
            .map_err(|e| WallSwitchError::DaemonError(name, e.to_string()))?;

        // 4. Wait for the process to appear in the system table using the shared System state.
        wait_for_process_ready(&mut sys, daemon.cmd_name, config)?;

        Ok(())
    }
}

/// Helper to instantiate a configured and refreshed `System` instance.
/// This centralizes the process table initialization parameters.
fn get_refreshed_system() -> System {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true, // remove_dead_processes
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    sys
}

/// Terminates all processes that match the provided name.
///
/// Safe and cross-platform. It silently ignores errors for processes
/// where permissions are denied or that have already terminated.
pub fn terminate_processes_by_name(name: &str) {
    if name.trim().is_empty() {
        return;
    }

    let sys = get_refreshed_system();
    let targets: Vec<_> = find_processes_by_name(&sys, name).collect();

    for process in targets {
        let _ = process.kill();
    }
}

/// Helper to find processes matching a given name.
/// This acts as the single source of truth for process matching logic.
fn find_processes_by_name<'a>(
    sys: &'a System,
    name: &'a str,
) -> impl Iterator<Item = &'a sysinfo::Process> {
    sys.processes().values().filter(move |process| {
        process.exe().is_some_and(|path| {
            path.file_name()
                .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(name))
        })
    })
}

/// Checks if a process is running using an existing, shared [`System`] instance.
///
/// This function avoids the resource overhead of instantiating a new system state
/// on every call, making it suitable for tight loops or high-frequency polling.
///
/// # Arguments
///
/// * `sys` - A mutable reference to a shared [`System`] state.
/// * `process_name` - The target process name to search for (case-insensitive).
///
/// # Returns
///
/// Returns `true` if a matching process is found, and `false` otherwise.
pub fn is_process_running_shared(sys: &mut System, process_name: &str) -> bool {
    // Only refresh executable paths to keep CPU footprint minimal during fast polls
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true, // remove_dead_processes
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    find_processes_by_name(sys, process_name).any(|_| true)
}

/// Checks if a process with the specified name is currently running.
///
/// This helper is a wrapper that handles its own temporary [`System`] context.
/// For performance-sensitive paths, use [`is_process_running_shared`] with an
/// external, reused state to avoid allocation overhead.
///
/// # Examples
///
/// ```no_run
/// use wallswitch::is_process_running;
///
/// if is_process_running("my-daemon") {
///     println!("Daemon is active.");
/// }
/// ```
pub fn is_process_running(process_name: &str) -> bool {
    let mut sys = System::new();
    is_process_running_shared(&mut sys, process_name)
}

/// Polls the system until the specified process is detected or the timeout is reached.
///
/// This implementation monitors the process list using a shared [`System`] state
/// to prevent high CPU utilization during tight polling loops.
/// Time tracking relies on [`Instant::now`] to bypass timer inaccuracies or drift
/// introduced by the OS scheduler during `sleep`.
///
/// # Arguments
///
/// * `sys` - A mutable reference to a shared [`System`] instance.
/// * `name` - The name of the process to wait for.
/// * `config` - Application configuration parameters.
///
/// # Errors
///
/// Returns a [`WallSwitchError::UnableToFind`] error if the process does not appear
/// within the 5-second window.
pub fn wait_for_process_ready(
    sys: &mut System,
    name: &str,
    config: &Config,
) -> WallSwitchResult<()> {
    let max_wait = Duration::from_secs_f32(5.0);
    let step = Duration::from_secs_f32(0.25);

    let start_time = Instant::now();

    while start_time.elapsed() < max_wait {
        sleep(step);

        // Query process presence using the shared System state
        if is_process_running_shared(sys, name) {
            if config.verbose {
                println!("\n{name} successfully initialized.");
            }
            return Ok(());
        }

        if config.verbose {
            let elapsed_secs = start_time.elapsed().as_secs_f32();
            print!(
                "\rWait to initialize {name}. Time: {elapsed_secs:0.2}/{:0.2}s",
                max_wait.as_secs_f32()
            );
            let _ = stdout().flush();
        }
    }

    if config.verbose {
        println!();
    }

    Err(WallSwitchError::UnableToFind(format!(
        "{name} daemon failed to respond after initialization."
    )))
}

/// Extension trait for `std::process::Command` to unify command execution logic.
pub trait CommandExt {
    /// Executes the command, printing output parameters in verbose mode or on failures.
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchError`] if the process fails to execute or exits with a non-zero status.
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

        let is_success = output.status.success();

        if !is_success || config.verbose {
            println!("\nprogram: {program:?}");
            println!("arguments: {arguments:#?}");

            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                println!("stdout:'{}'\n", stdout.trim());
            }
        }

        if !is_success {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let status = output.status;

            eprintln!("{context} status: {status}");
            eprintln!("{context} stderr: {stderr}");

            return Err(WallSwitchError::CommandFailed {
                program: format!("{program:?}"),
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
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_is_process_running_with_invalid_name() {
        // Ensures that a completely randomized name returns false.
        assert!(!is_process_running("non_existent_process_xyz_123"));
    }

    #[test]
    fn test_command_ext_success() {
        let config = Config {
            dry_run: false,
            verbose: false,
            ..Default::default()
        };

        // Standard commands like 'cargo' or 'echo' are generally available.
        #[cfg(target_os = "windows")]
        let mut cmd = Command::new("cmd");
        #[cfg(target_os = "windows")]
        cmd.args(["/C", "echo hello"]);

        #[cfg(not(target_os = "windows"))]
        let mut cmd = Command::new("echo");
        #[cfg(not(target_os = "windows"))]
        cmd.arg("hello");

        let result = cmd.run_with_config(&config, "Test Context");
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_ext_failure() {
        let config = Config {
            dry_run: false,
            verbose: false,
            ..Default::default()
        };

        // Attempts to run a non-existent binary to trigger failure flow.
        let mut cmd = Command::new("non_existent_binary_for_test");
        let result = cmd.run_with_config(&config, "Test Fail Context");
        assert!(result.is_err());
    }
}
