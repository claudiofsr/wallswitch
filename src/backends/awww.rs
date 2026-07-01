use crate::{
    CommandExt, Config, DaemonConfig, DaemonManager, FileInfo, WallSwitchResult, WallpaperBackend,
    detect_monitors, get_random_integer,
};
use std::process::Command;

/// Backend implementing dynamic wallpaper transitions on Wayland via `awww`.
pub struct AwwwBackend;

impl WallpaperBackend for AwwwBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Declarative configuration: The backend only knows WHAT to run,
        // not HOW to manage the lifecycle.
        let daemon_cfg = DaemonConfig {
            cmd_name: "awww-daemon",
            pre_spawn_hook: Some(clean_stale_sockets),
        };

        // Logic is DRY: DaemonManager handles kill, hook, spawn, and polling.
        DaemonManager::ensure_running(config, &daemon_cfg)?;

        for (image, monitor) in images.iter().cycle().zip(monitors.iter()) {
            let effect = get_transition_effect(config);

            let mut wall_cmd = Command::new("awww");
            wall_cmd
                .args(["img", "-o", monitor])
                .arg(&image.path)
                .args(["--transition-type", &effect])
                .args([
                    "--transition-duration",
                    &config.transition_duration.to_string(),
                ])
                .args(["--transition-fps", &config.transition_fps.to_string()])
                .args(["--transition-angle", &config.transition_angle.to_string()])
                .args(["--transition-pos", &config.transition_pos]);

            if config.dry_run {
                println!("[DRY-RUN] Would execute: {:?}", wall_cmd);
            } else {
                wall_cmd.run_with_config(config, &format!("Apply awww on {monitor}"))?;
            }
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
fn clean_stale_sockets() -> WallSwitchResult<()> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains("awww") && name.ends_with(".sock") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    Ok(())
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
