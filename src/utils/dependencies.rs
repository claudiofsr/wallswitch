use crate::{WallSwitchError, WallSwitchResult};
use std::path::PathBuf;

/// Helper to check if a command exists in the system PATH.
pub fn is_installed(binary: &str) -> bool {
    which::which(binary).is_ok()
}

/// Locate the exact absolute path for a system binary.
pub fn where_is(cmd: &str) -> WallSwitchResult<PathBuf> {
    which::which(cmd).map_err(|_| WallSwitchError::UnableToFind(cmd.to_string()))
}

/// Get the 'feh' binary path, used by the Openbox/X11 backend.
pub fn get_feh_path(verbose: bool) -> WallSwitchResult<PathBuf> {
    match where_is("feh") {
        Ok(feh_path) => {
            if verbose {
                println!("feh found at: {}", feh_path.display());
            }
            Ok(feh_path)
        }
        Err(_) => Err(WallSwitchError::UnableToFind("feh".to_string())),
    }
}

/// Get the 'awww' binary path, used by the generic Wayland backend.
pub fn get_awww_path(verbose: bool) -> WallSwitchResult<PathBuf> {
    match where_is("awww") {
        Ok(awww_path) => {
            if verbose {
                println!("awww daemon found at: {}", awww_path.display());
            }
            Ok(awww_path)
        }
        Err(_) => {
            let install_instructions = "\
                The 'awww' wallpaper daemon was not found on your system.\n\n\
                Please install it to enable modern Wayland transitions:\n\n\
                - Arch Linux / Manjaro (AUR):\n    \
                    paru -S awww  (or yay -S awww)\n\n\
                - Debian / Ubuntu:\n    \
                    Download the latest .deb from the GitHub releases page.\n\n\
                - Fedora / RPM-based:\n    \
                    Download the latest .rpm from the GitHub releases page.\n\n\
                - Compile from Source (Any distro):\n    \
                    git clone https://codeberg.org/LGFae/awww.git\n    \
                    cd awww\n    \
                    cargo build --release\n    \
                    cargo install --path .\n"
                .to_string();

            Err(WallSwitchError::UnableToFind(install_instructions))
        }
    }
}
