use crate::{
    CommandExt, Config, DaemonConfig, DaemonManager, FileInfo, WallSwitchError, WallSwitchResult,
    WallpaperBackend, detect_monitors,
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

        // Define the lifecycle configuration for the hyprpaper daemon
        let daemon_cfg = DaemonConfig {
            name: "hyprpaper",
            spawn_cmd: "hyprpaper",
            kill_cmd: Some("hyprpaper"),
        };

        // Ensure daemon is running using the centralized manager
        DaemonManager::ensure_running(config, &daemon_cfg, || {
            Command::new(daemon_cfg.spawn_cmd)
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
        })?;

        // Check which images are already preloaded
        let mut check_cmd = Command::new("hyprctl");
        check_cmd.args(["hyprpaper", "listloaded"]);

        let loaded_str = match check_cmd.run_with_config(config, "Check loaded hyprpaper images")? {
            out if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
            _ => {
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

            // Preload the image if it's not already in the loaded list
            if !loaded_str.contains(path_str) {
                let mut preload_cmd = Command::new("hyprctl");
                preload_cmd.args(["hyprpaper", "preload", path_str]);

                // We ignore the result here as per original behavior,
                // but run_with_config handles the verbosity/dry_run logging.
                let _ = preload_cmd.run_with_config(config, &format!("Preload {path_str}"));
            }

            // Apply the wallpaper to the specific monitor
            let mut wall_cmd = Command::new("hyprctl");
            let wall_arg = format!("{monitor},{path_str}");
            wall_cmd.args(["hyprpaper", "wallpaper", &wall_arg]);

            wall_cmd.run_with_config(config, &format!("Apply wallpaper on {monitor}"))?;
        }

        // Cleanup unused wallpapers
        let mut unload_cmd = Command::new("hyprctl");
        unload_cmd.args(["hyprpaper", "unload", "unused"]);
        let _ = unload_cmd.run_with_config(config, "Unload unused hyprpaper images");

        Ok(())
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_hyprpaper_backend {
    use crate::is_process_running;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_process_running("hyprpaper");
    }
}
