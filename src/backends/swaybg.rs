use crate::{
    Config, FileInfo, WallSwitchError, WallSwitchResult, WallpaperBackend, backends,
    detect_monitors,
};
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

/// Backend implementing static wallpaper rendering on Wayland via `swaybg`.
pub struct SwaybgBackend;

impl WallpaperBackend for SwaybgBackend {
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}\n");
        }

        // Starts or restarts daemon if necessary
        ensure_daemon_running(config)?;

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
            let arguments: Vec<_> = cmd.get_args().collect::<Vec<_>>();
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

// ==============================================================================
// INTERNAL HELPERS
// ==============================================================================

fn is_daemon_alive() -> bool {
    backends::is_process_running("swaybg")
}

fn ensure_daemon_running(config: &Config) -> WallSwitchResult<()> {
    // swaybg lacks dynamic IPC reloading, so we must terminate previous running instances
    // and spawn a new daemon with updated paths to reflect the new wallpaper.
    if is_daemon_alive() {
        if config.dry_run {
            println!("[DRY-RUN] Would terminate previous swaybg instances.");
        } else {
            let _ = Command::new("pkill").arg("swaybg").output();
            // Wait briefly to allow the kernel to clean up the terminated process
            thread::sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_swaybg_backend {
    use super::*;

    #[test]
    fn test_is_daemon_alive_on_idle() {
        let _ = is_daemon_alive();
    }
}
