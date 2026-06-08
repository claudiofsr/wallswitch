use crate::{
    Config, FileInfo, WallSwitchError, WallSwitchResult, WallpaperBackend, backends,
    detect_monitors,
};
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Backend implementing static wallpaper rendering on Wayland via `swaybg`.
pub struct SwaybgBackend;

impl WallpaperBackend for SwaybgBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Starts or restarts daemon if necessary
        ensure_daemon_running(config)?;

        let mut cmd = Command::new("swaybg");
        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image.path.to_str().unwrap_or_default();
            cmd.arg("-o")
                .arg(monitor)
                .arg("-i")
                .arg(path_str)
                .arg("-m")
                .arg("fill");
        }

        if config.verbose {
            let program = cmd.get_program();
            let arguments: Vec<_> = cmd.get_args().collect::<Vec<_>>();
            println!("\nprogram: {program:?}");
            println!("arguments: {arguments:#?}");
        }

        if config.dry_run {
            println!("[DRY-RUN] Would spawn swaybg daemon: {:?}", cmd);
        } else {
            // Note: Since swaybg blocks while displaying the image, we spawn the process
            // in the background rather than running it synchronously via output-based execution helpers.
            cmd.stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(WallSwitchError::Io)?;
        }

        Ok(())
    }
}

// ==============================================================================
// INTERNAL HELPERS
// ==============================================================================

/// Helper status check to verify if the background process is active.
fn is_daemon_alive() -> bool {
    backends::is_process_running("swaybg")
}

/// Standardized termination and cleanup handler for swaybg.
///
/// Unlike backends with a dynamic configuration interface (e.g., IPC),
/// `swaybg` must be terminated and spawned anew to apply updated configuration properties.
fn ensure_daemon_running(config: &Config) -> WallSwitchResult<()> {
    if is_daemon_alive() {
        if config.dry_run {
            println!("[DRY-RUN] Would terminate previous swaybg instances.");
        } else {
            let _ = Command::new("killall").arg("swaybg").output();

            // Wait a brief moment to ensure the OS terminates the process and frees resources
            thread::sleep(Duration::from_millis(150));
        }
    }
    Ok(())
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_swaybg_backend {
    use super::*;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_daemon_alive();
    }
}
