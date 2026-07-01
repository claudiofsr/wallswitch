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
        let daemon_cfg = DaemonConfig {
            cmd_name: "swaybg",
            pre_spawn_hook: None,
        };

        DaemonManager::ensure_running(config, &daemon_cfg)?;

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
