use crate::{
    CommandExt, Config, DaemonConfig, DaemonManager, FileInfo, WallSwitchError, WallSwitchResult,
    WallpaperBackend, detect_monitors, get_random_integer,
};
use std::{
    env, fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Backend implementing dynamic wallpaper transitions on Wayland via `awww`.
pub struct AwwwBackend;

impl WallpaperBackend for AwwwBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Define the lifecycle configuration for the aww daemon
        let daemon_cfg = DaemonConfig {
            name: "awww-daemon",
            spawn_cmd: "awww-daemon",
            kill_cmd: Some("awww-daemon"),
        };

        // Ensure daemon is running using the centralized manager
        DaemonManager::ensure_running(config, &daemon_cfg, || {
            // aww-specific: clean stale sockets before spawning
            clean_stale_sockets();

            Command::new(daemon_cfg.spawn_cmd)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| WallSwitchError::AwwwDaemonError(e.to_string()))?;

            // Allow a brief initialization window for socket setup
            thread::sleep(Duration::from_millis(300));
            Ok(())
        })?;

        // Cycle through images to ensure all monitors receive a command,
        // avoiding issues when detected monitors > configured monitors.
        for (image, monitor) in images.iter().cycle().zip(monitors.iter()) {
            let effect = get_transition_effect(config);

            let mut cmd = Command::new("awww");
            cmd.args(["img", "-o", monitor])
                .arg(&image.path)
                .args(["--transition-type", &effect])
                .args([
                    "--transition-duration",
                    &config.transition_duration.to_string(),
                ])
                .args(["--transition-fps", &config.transition_fps.to_string()])
                .args(["--transition-angle", &config.transition_angle.to_string()])
                .args(["--transition-pos", &config.transition_pos]);

            // Use the CommandExt trait for idiomatic execution
            cmd.run_with_config(config, &format!("Apply awww on {monitor}"))?;
        }

        Ok(())
    }
}

// ==============================================================================
// INTERNAL HELPERS
// ==============================================================================

/// Evaluates and selects the transition effect based on current configuration.
fn get_transition_effect(config: &Config) -> String {
    if config.transition_type.to_lowercase() == "random" {
        let effects = ["wipe", "fade", "center", "outer", "wave", "left", "right"];
        let idx: usize = get_random_integer(0, effects.len() - 1);
        effects[idx].to_string()
    } else {
        config.transition_type.clone()
    }
}

/// Cleans orphaned local domain sockets to prevent connectivity locks.
/// This is specific to the `awww` backend requirements.
fn clean_stale_sockets() {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("awww") && name.ends_with(".sock") {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_awww_backend {
    use crate::is_process_running;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_process_running("awww-daemon");
    }
}
