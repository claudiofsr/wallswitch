use crate::{
    Config, DaemonConfig, DaemonManager, FileInfo, WallSwitchError, WallSwitchResult,
    WallpaperBackend, detect_monitors,
};
use std::process::{Command, Stdio};

/// Backend implementing static wallpaper rendering on Wayland via `swaybg`.
pub struct SwaybgBackend;

impl WallpaperBackend for SwaybgBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Define the lifecycle configuration for the swaybg daemon.
        // Unlike backends with dynamic configuration interfaces (e.g., IPC),
        // `swaybg` must be terminated and spawned anew to apply updated properties.
        let daemon_cfg = DaemonConfig {
            name: "swaybg",
            spawn_cmd: "swaybg",
            kill_cmd: Some("swaybg"),
        };

        // Ensure daemon is running using the centralized manager.
        // This handles the killall and the 150ms sleep automatically.
        DaemonManager::ensure_running(config, &daemon_cfg, || {
            Command::new(daemon_cfg.spawn_cmd)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(WallSwitchError::Io)?;
            Ok(())
        })?;

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
            let arguments: Vec<_> = cmd.get_args().collect();
            println!("\nprogram: {program:?}");
            println!("arguments: {arguments:#?}");
        }

        if config.dry_run {
            println!("[DRY-RUN] Would spawn swaybg daemon: {:?}", cmd);
        } else {
            // Note: Since swaybg blocks while displaying the image, we spawn the process
            // in the background rather than running it synchronously via output-based
            // execution helpers (which would hang the program).
            cmd.stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(WallSwitchError::Io)?;
        }

        Ok(())
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_swaybg_backend {
    use crate::is_process_running;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_process_running("swaybg");
    }
}
