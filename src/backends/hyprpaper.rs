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
                .map_err(|e| WallSwitchError::AwwwDaemonError(e.to_string()))?;

            // Allow a brief initialization window for socket setup
            thread::sleep(Duration::from_millis(300));
            Ok(())
        })?;

        // Cycle through images and apply them to the detected monitors.
        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image.path.to_str().unwrap_or_default();

            // Aplicar o wallpaper
            let mut wall_cmd = Command::new("hyprctl");
            let wall_arg = format!("{monitor},{path_str}");
            wall_cmd.args(["hyprpaper", "wallpaper", &wall_arg]);

            if config.dry_run {
                println!("[DRY-RUN] Would execute: {:?}", wall_cmd);
            } else {
                // Se o comando de aplicar falhar aqui, o programa para,
                // pois é a ação principal.
                wall_cmd.run_with_config(config, &format!("Apply wallpaper on {monitor}"))?;
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
