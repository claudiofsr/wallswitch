use crate::{WallSwitchError, WallSwitchResult, exec_cmd};
use std::{path::PathBuf, process::Command};

/// Get the magick binary path
pub fn get_magick_path(verbose: bool) -> Result<PathBuf, WallSwitchError> {
    let path_magick = where_is("magick", verbose);

    if let Ok(magick) = path_magick
        && magick.is_file()
    {
        return Ok(magick);
    };

    let path_convert = where_is("convert", verbose);

    if let Ok(convert) = path_convert
        && convert.is_file()
    {
        return Ok(convert);
    };

    Err(WallSwitchError::UnableToFind("magick".to_string()))
}

/// Get the feh binary path
pub fn get_feh_path(verbose: bool) -> Result<PathBuf, WallSwitchError> {
    let path_feh = where_is("feh", verbose);

    match path_feh {
        Ok(feh) if feh.is_file() => Ok(feh),
        _ => Err(WallSwitchError::UnableToFind("feh".to_string())),
    }
}

/// Locate the binary path for a command
///
/// Example:
///
/// whereis -b magick
pub fn where_is(cmd: &str, verbose: bool) -> WallSwitchResult<PathBuf> {
    let mut whereis = Command::new("whereis");
    let whereis_cmd = whereis.args(["-b", cmd]);
    let output = exec_cmd(whereis_cmd, verbose, "whereis")?;
    bytes_to_path(cmd, &output.stdout)
}

/// Convert a byte slice to pathbuf
/// Returns a WallSwitchResult<PathBuf> which will be an error if the path is not found.
fn bytes_to_path(cmd: &str, bytes: &[u8]) -> Result<PathBuf, WallSwitchError> {
    let path_str = String::from_utf8_lossy(bytes);

    // Find the path string
    let found_path_opt = path_str
        .split(['\n', ' '])
        .find(|s| !s.is_empty() && s.contains("bin") && s.contains(cmd));

    match found_path_opt {
        Some(s) => {
            let path = PathBuf::from(s);
            if path.is_file() {
                // Also verify that the found path actually points to a file
                Ok(path)
            } else {
                Err(WallSwitchError::UnableToFind(format!(
                    "'{}' found at '{}' but it's not a file",
                    cmd, s
                )))
            }
        }
        None => Err(WallSwitchError::UnableToFind(format!(
            "'{}' not found in system paths",
            cmd
        ))),
    }
}
