use crate::{
    Config, FileInfo, WallSwitchError, WallSwitchResult, WallpaperBackend, backends,
    detect_monitors, exec_cmd, get_random_integer,
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

        // Starts or restarts daemon if necessary
        ensure_daemon_running(config)?;

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

            if config.dry_run {
                println!("[DRY-RUN] Would execute: {:?}", cmd);
            } else {
                exec_cmd(
                    &mut cmd,
                    config.verbose,
                    &format!("Apply awww on {}", monitor),
                )?;
            }
        }

        Ok(())
    }
}

// ==============================================================================
// INTERNAL HELPERS
// ==============================================================================

fn get_transition_effect(config: &Config) -> String {
    if config.transition_type.to_lowercase() == "random" {
        let effects = ["wipe", "fade", "center", "outer", "wave", "left", "right"];
        let idx: usize = get_random_integer(0, effects.len() - 1);
        effects[idx].to_string()
    } else {
        config.transition_type.clone()
    }
}

fn is_daemon_alive() -> bool {
    backends::is_process_running("awww-daemon")
}

fn ensure_daemon_running(config: &Config) -> WallSwitchResult<()> {
    backends::ensure_background_daemon(config, "awww-daemon", is_daemon_alive, || {
        // Send termination signal to any existing daemon
        let _ = Command::new("killall").arg("awww-daemon").output();

        // Wait briefly to allow the kernel to clean up the terminated process
        thread::sleep(Duration::from_millis(150));

        // Remove stale socket files safely
        clean_stale_sockets();

        Command::new("awww-daemon")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| WallSwitchError::AwwwDaemonError(e.to_string()))?;

        // Allow a brief initialization window for socket setup
        thread::sleep(Duration::from_millis(300));
        Ok(())
    })
}

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
    use super::*;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_daemon_alive();
    }
}
