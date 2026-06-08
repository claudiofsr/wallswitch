//! Process management and instance serialization utilities using `sysinfo`.
//!
//! This module ensures that only a single instance of the `wallswitch` application
//! runs concurrently on the system. It leverages the cross-platform `sysinfo` crate
//! to safely and efficiently query active processes across Linux, macOS, and BSD.

use crate::{Config, Environment, WallSwitchResult};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Scans the system for other running instances of `wallswitch` and terminates them.
///
/// This function acts as an initialization guard. It retrieves all active PIDs
/// matching the binary name, filters out the current process's PID, and issues a
/// terminal signal `kill -9` to stop any stale background runs.
///
/// # Errors
///
/// Returns a [`WallSwitchError`](crate::WallSwitchError) if environment variables
/// cannot be read, or if termination signals fail to send.
pub fn kill_other_instances(config: &Config) -> WallSwitchResult<()> {
    if config.dry_run {
        if config.verbose {
            println!("[DRY-RUN] Skipping killing other instances.");
        }
        return Ok(());
    }

    let env = Environment::new()?;
    let pkg_name = env.get_pkg_name();

    let current_pid: u32 = std::process::id();
    let pids: Vec<u32> = get_pids(pkg_name, config)?;

    for pid in pids {
        if pid != current_pid {
            if config.verbose {
                println!("Killing previous instances: kill -9 {pid}\n");
            }
            kill_app(pid, config)?;
        }
    }

    Ok(())
}

/// Discovers running process IDs (PIDs) matching the package name.
///
/// Queries active system processes using `sysinfo`. This provides native, safe,
/// and cross-platform process discovery without spawning external pgrep/pidof shells.
fn get_pids(pkg_name: &str, config: &Config) -> WallSwitchResult<Vec<u32>> {
    let mut pids = Vec::new();
    let current_pid = std::process::id();

    // Initialize System struct
    let mut sys = System::new();

    // Refresh only process executable information to minimize performance overhead.
    // The second argument 'true' ensures dead processes are cleared from the internal cache.
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true, // remove_dead_processes
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    for (pid, process) in sys.processes() {
        let pid_u32 = pid.as_u32();
        if pid_u32 == current_pid {
            continue;
        }

        // Compare the executable's filename with our package name
        if let Some(exe_path) = process.exe()
            && let Some(name) = exe_path.file_name().and_then(|n| n.to_str())
            && name == pkg_name
        {
            pids.push(pid_u32);
        }
    }

    pids.sort();
    pids.dedup();

    if !pids.is_empty() && config.verbose {
        println!("Process identification (pid) found via sysinfo:");
        println!("pids: {pids:?}\n");
    }

    Ok(pids)
}

/// Terminates a process natively by its PID using `sysinfo`.
fn kill_app(pid_number: u32, config: &Config) -> WallSwitchResult<()> {
    let pid = Pid::from_u32(pid_number);
    let mut sys = System::new();

    // Verify if the specific process is still alive before sending the signal
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);

    if let Some(process) = sys.process(pid) {
        if config.verbose {
            println!("Sending native termination signal to PID: {pid_number}");
        }
        process.kill();
    }

    Ok(())
}

#[cfg(test)]
mod test_pids {
    use super::*;

    #[test]
    fn test_current_pid_exclusion() {
        let config = Config::default();
        let env = Environment::fallback();
        let pids = get_pids(env.get_pkg_name(), &config).unwrap();
        let current_pid = std::process::id();
        assert!(!pids.contains(&current_pid));
    }
}
