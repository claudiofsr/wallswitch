use crate::{
    Config, FileInfo, WallSwitchError, WallSwitchResult, WallpaperBackend, detect_monitors,
    exec_cmd, get_random_integer,
};
use std::{
    env, fs,
    io::{self, Write},
    process::{Command, Stdio},
    thread::sleep,
    time::Duration,
};

pub struct AwwwBackend;

impl WallpaperBackend for AwwwBackend {
    fn build_commands(_images: &[FileInfo], _config: &Config) -> WallSwitchResult<Vec<Command>> {
        // Overrides `apply` below directly since we need daemon state management.
        Ok(vec![])
    }

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
        let idx = get_random_integer(0, (effects.len() - 1) as u64) as usize;
        effects[idx].to_string()
    } else {
        config.transition_type.clone()
    }
}

fn ensure_daemon_running(config: &Config) -> WallSwitchResult<()> {
    if is_daemon_alive() {
        return Ok(());
    }

    if config.verbose {
        println!("awww-daemon is down. Performing clean start...");
    }

    let _ = Command::new("killall").arg("awww-daemon").output();
    clean_stale_sockets();

    Command::new("awww-daemon")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| WallSwitchError::AwwwDaemonError(e.to_string()))?;

    let mut elapsed = 0.0;
    let step = 0.2;
    let max_wait = 5.0;

    while elapsed < max_wait {
        if is_daemon_alive() {
            if config.verbose {
                println!("\nawww-daemon successfully initialized.");
            }
            return Ok(());
        }

        if config.verbose {
            print!(
                "\rWait to initialize awww-daemon. Time: {:0.1}/{:0.1}",
                elapsed, max_wait
            );
            io::stdout().flush().ok();
        }

        sleep(Duration::from_secs_f32(step));
        elapsed += step;
    }

    if config.verbose {
        println!();
    }

    Err(WallSwitchError::AwwwDaemonError(
        "Daemon failed to initialize.".into(),
    ))
}

fn is_daemon_alive() -> bool {
    Command::new("awww")
        .arg("query")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
