use crate::{
    CommandExt, Config, DaemonConfig, DaemonManager, FileInfo, WallSwitchError, WallSwitchResult,
    WallpaperBackend, detect_monitors,
};
use std::process::Command;

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
            cmd_name: "hyprpaper",
            pre_spawn_hook: None,
        };

        DaemonManager::ensure_running(config, &daemon_cfg)?;

        // Cycle through images and apply them to the detected monitors.
        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image
                .path
                .to_str()
                .ok_or_else(|| WallSwitchError::InvalidFilename(image.path.clone()))?;

            let mut wall_cmd = Command::new("hyprctl");
            let wall_arg = format!("{monitor},{path_str}");
            wall_cmd.args(["hyprpaper", "wallpaper", &wall_arg]);

            if config.dry_run {
                println!("[DRY-RUN] Would execute: {:?}", wall_cmd);
            } else {
                wall_cmd.run_with_config(config, &format!("Apply hyprpaper on {monitor}"))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_hyprpaper_backend {
    use crate::is_process_running;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_process_running("hyprpaper");
    }
}
