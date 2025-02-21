use crate::{exec_cmd, MyResult, WSError};
use std::{path::PathBuf, process::Command};

/// Get the magick binary path
pub fn get_magick_path(verbose: bool) -> Result<PathBuf, WSError<'static>> {
    let path_magick = where_is("magick", verbose);

    if let Ok(magick) = path_magick {
        if magick.is_file() {
            return Ok(magick);
        }
    };

    let path_convert = where_is("convert", verbose);

    if let Ok(convert) = path_convert {
        if convert.is_file() {
            return Ok(convert);
        }
    };

    Err(WSError::UnableToFind("magick"))
}

/// Get the feh binary path
pub fn get_feh_path(verbose: bool) -> Result<PathBuf, WSError<'static>> {
    let path_feh = where_is("feh", verbose);

    match path_feh {
        Ok(feh) if feh.is_file() => Ok(feh),
        _ => Err(WSError::UnableToFind("feh")),
    }
}

/// Locate the binary path for a command
///
/// Example:
///
/// whereis -b magick
pub fn where_is(cmd: &str, verbose: bool) -> MyResult<PathBuf> {
    let mut whereis = Command::new("whereis");
    let whereis_cmd = whereis.args(["-b", cmd]);
    let output = exec_cmd(whereis_cmd, verbose, "whereis")?;
    Ok(bytes_to_path(cmd, &output.stdout))
}

/// Convert a byte slice to pathbuf
fn bytes_to_path(cmd: &str, bytes: &[u8]) -> PathBuf {
    String::from_utf8_lossy(bytes)
        .split(['\n', ' '])
        .find(|s| s.contains("bin") && s.contains(cmd))
        .into_iter()
        .collect()
}
