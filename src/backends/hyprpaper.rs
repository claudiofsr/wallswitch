use crate::{
    Config, FileInfo, WallSwitchError, WallSwitchResult, WallpaperBackend, backends,
    detect_monitors, exec_cmd,
};
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Backend implementing wallpaper orchestration for the Hyprland compositor via `hyprpaper`.
pub struct HyprlandBackend;

impl WallpaperBackend for HyprlandBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Starts or restarts daemon if necessary
        ensure_daemon_running(config)?;

        let mut check_cmd = Command::new("hyprctl");
        check_cmd.args(["hyprpaper", "listloaded"]);

        let loaded_str = match check_cmd.output() {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => {
                if config.dry_run {
                    "[DRY-RUN] hyprpaper daemon is offline".to_string()
                } else {
                    return Err(WallSwitchError::UnableToFind(
                        "hyprpaper daemon not running".into(),
                    ));
                }
            }
        };

        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image.path.to_str().unwrap_or_default();

            if !loaded_str.contains(path_str) {
                let mut preload_cmd = Command::new("hyprctl");
                preload_cmd.args(["hyprpaper", "preload", path_str]);

                if config.verbose {
                    println!("\nprogram: {:?}", preload_cmd.get_program());
                    println!(
                        "arguments: {:#?}",
                        preload_cmd.get_args().collect::<Vec<_>>()
                    );
                }
                if config.dry_run {
                    println!("[DRY-RUN] Would execute: {:?}", preload_cmd);
                } else {
                    let _ = preload_cmd.output();
                }
            }

            let mut wall_cmd = Command::new("hyprctl");
            let wall_arg = format!("{monitor},{path_str}");
            wall_cmd.args(["hyprpaper", "wallpaper", &wall_arg]);

            if config.dry_run {
                println!("[DRY-RUN] Would execute: {:?}", wall_cmd);
            } else {
                exec_cmd(
                    &mut wall_cmd,
                    config.verbose,
                    &format!("Apply wallpaper on {monitor}"),
                )?;
            }
        }

        let mut unload_cmd = Command::new("hyprctl");
        unload_cmd.args(["hyprpaper", "unload", "unused"]);
        if config.dry_run {
            println!("[DRY-RUN] Would execute: {:?}", unload_cmd);
        } else {
            let _ = unload_cmd.output();
        }

        Ok(())
    }
}

// ==============================================================================
// INTERNAL HELPERS
// ==============================================================================

/// Helper status check to verify if the background process is active.
fn is_daemon_alive() -> bool {
    backends::is_process_running("hyprpaper")
}

/// Standardized coordinator to clean stale environments and safely spin up the background process.
fn ensure_daemon_running(config: &Config) -> WallSwitchResult<()> {
    backends::ensure_background_daemon(config, "hyprpaper", is_daemon_alive, || {
        // Kill any existing/stale hyprpaper instances to avoid socket conflicts
        let _ = Command::new("killall").arg("hyprpaper").output();

        // Wait a brief moment to ensure the OS terminates the process and frees the socket
        thread::sleep(Duration::from_millis(150));

        Command::new("hyprpaper")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| {
                WallSwitchError::UnableToFind(format!(
                    "Failed to spawn hyprpaper background daemon. Is it installed? Details: {err}"
                ))
            })?;

        // Allow a brief initialization window for socket/EGL context setup
        thread::sleep(Duration::from_millis(300));
        Ok(())
    })
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_hyprpaper_backend {
    use super::*;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_daemon_alive();
    }
}
