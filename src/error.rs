use crate::{Colors, Dimension, Orientation};
use image::ImageError;
use std::{io, num::TryFromIntError, path::PathBuf, string::FromUtf8Error};
use thiserror::Error;

/**
Result type to simplify function signatures.

This is a custom result type that uses our custom `WallSwitchError` for the error type.

Functions can return `WallSwitchResult<T>` and then use `?` to automatically propagate errors.
*/
pub type WallSwitchResult<T> = Result<T, WallSwitchError>;

/// WallSwitch Error enum
///
/// The `WallSwitchError` enum defines the error values.
///
/// <https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html>
#[derive(Error, Debug)]
pub enum WallSwitchError {
    /// Error for command-line arguments that have invalid values.
    #[error(
        "{e}: '{a}' value '{v}' must be at least '{n}'. \
        The condition ({v} >= {n}) is false.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        v = value.yellow(),
        a = arg.yellow(),
        n = num.green(),
        h = "--help".green(),
    )]
    AtLeastValue {
        arg: String,
        value: String,
        num: u64,
    },

    /// Error for command-line arguments that have values exceeding a maximum.
    #[error(
        "{e}: '{a}' value '{v}' must be at most '{n}'. \
        The condition ({v} <= {n}) is false.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        v = value.yellow(),
        a = arg.yellow(),
        n = num.green(),
        h = "--help".green(),
    )]
    AtMostValue {
        arg: String,
        value: String,
        num: u64,
    },

    /// Error for failed command execution instead of panicking.
    #[error(
        "{e}: Command '{program}' failed!\n\
        Status: {status}\n\
        Error details: {stderr}",
        e = "Error".red().bold(),
    )]
    CommandFailed {
        program: String,
        status: String,
        stderr: String,
    },

    /// Error when a wallpaper image is corrupt or cannot be decoded.
    #[error(
        "Error: failed to decode or open image.\n\
        Details: {source}\n\
        Disregard the path: '{path}'",
        path = .path.display().to_string().yellow(),
    )]
    CorruptImage {
        path: PathBuf,
        #[source]
        source: ImageError,
    },

    /// Error when the some daemon fails to initialize or respond.
    #[error("{0} daemon failed to start or is unresponsive: {1}")]
    DaemonError(String, String),

    /// Error when desktop environment detection fails completely.
    #[error("{e}: Could not detect desktop environment. Please set DESKTOP_SESSION manually.", e = "Error".red().bold())]
    DesktopDetectionFailed,

    /// Error when an image path should be disregarded.
    #[error("Disregard the path: '{p}'.", p = .0.display().yellow(),)]
    DisregardPath(PathBuf),

    /// Error when trying to select or sample from an empty slice.
    #[error("{e}: Cannot select an item because the collection is empty.", e = "Error".red().bold())]
    EmptySlice,

    /// Error when a required system environment variable is missing.
    #[error("{e}: Environment variable '{0}' not found.", e = "Error".red().bold())]
    EnvVarMissing(String),

    /// Error when failing to convert byte output to a UTF-8 string.
    #[error("Failed to convert command output to UTF-8: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    /// Error propagated from the image processing library.
    #[error("Image library error: {0}")]
    Image(#[from] ImageError),

    /// Error when image dimensions are invalid.
    #[error("{0}")]
    InvalidDimension(#[from] DimensionError),

    /// Error for file paths containing invalid filenames.
    #[error(
        "{e}: Invalid file name --> Disregard the path: '{p}'.",
        e = "Error".red().bold(),
        p = .0.display().yellow(),
    )]
    InvalidFilename(PathBuf),

    /// Error for image file sizes that are outside the allowed range.
    #[error(
        "{e}: invalid file size '{s}' bytes. \
        The condition ({min} <= {s} <= {max}) is false.",
        e = "Error".red().bold(),
        min = min_size.green(),
        max = max_size.green(),
        s = size.yellow(),
    )]
    InvalidSize {
        min_size: u64,
        size: u64,
        max_size: u64,
    },

    /// Error for command-line arguments containing invalid values.
    #[error(
        "{e}: invalid value '{v}' for '{a}'.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        v = value.yellow(),
        a = arg.green(),
        h = "--help".green(),
    )]
    InvalidValue { arg: String, value: String },

    /// Standard I/O error wrapper.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error for I/O operations associated with a specific file path.
    #[error("{e}: Failed to create file {path:?}\n{io_error}", e = "Error".red().bold())]
    IOError {
        path: PathBuf,
        #[source]
        io_error: io::Error,
    },

    /// Error when a JSON serialization or deserialization operation fails.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error when obtaining the maximum valid value for a parameter fails.
    #[error("Unable to obtain maximum value!")]
    MaxValue,

    /// Error when obtaining the minimum valid value for a parameter fails.
    #[error("Unable to obtain minimum value!")]
    MinValue,

    /// Error for command-line arguments missing required values.
    #[error(
        "{e}: missing value for '{a}'.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        a = arg.yellow(),
        h = "--help".green(),
    )]
    MissingValue { arg: String },

    /// Error when the minimum value is configured greater than the maximum value.
    #[error(
        "{e}: min ({min}) must be less than or equal to max ({max})\n\
        The condition ({min} <= {max}) is false.",
        e = "Error".red().bold()
    )]
    MinMax { min: u64, max: u64 },

    /// Error when no supported Wayland wallpaper utility (swaybg, hyprpaper, awww) is found.
    #[error(
        "{e}: No Wayland wallpaper utility was found on your system.\n\n\
        To fix this, please install at least one of them:\n\
        - Manjaro/Arch: {pacman}\n\
        - Fedora: {dnf}\n\
        - Debian/Ubuntu: {apt}",
        e = "Error".red().bold(),
        pacman = "sudo pacman -S awww swaybg hyprpaper".green(),
        dnf = "sudo dnf install awww swaybg hyprpaper".green(),
        apt = "sudo apt install awww swaybg hyprpaper".green(),
    )]
    MissingWaylandTools,

    /// Error when no valid images are found in the specified directories.
    #[error(
        "{e}: no images found in image directories!\n\
        directories: {paths:#?}",
        e = "Error".red().bold(),
    )]
    NoImages { paths: Vec<PathBuf> },

    /// Error when no active monitors are detected by the system tool.
    #[error(
        "{e}: no active monitors detected via '{tool}'!",
        e = "Error".red().bold(),
        tool = .0.yellow(),
    )]
    NoMonitors(String),

    /// Error for an invalid image orientation configuration.
    #[error(
        "invalid orientation.\n\n\
        Valid options: [{h}, {v}]",
        h = Orientation::Horizontal.green(),
        v = Orientation::Vertical.green(),
    )]
    InvalidOrientation,

    /// Error when the number of available image files is insufficient.
    #[error(
        "{e}: insufficient number of image files!\n\
        Found only {n} image file(s):\n\
        {paths:#?}",
        n = nfiles.yellow(),
        e = "Error".red().bold(),
    )]
    InsufficientImages { paths: Vec<PathBuf>, nfiles: usize },

    /// Error when an insufficient number of valid images are found.
    #[error("Insufficient number of valid images!")]
    InsufficientNumber,

    /// Error when the specified wallpaper directory does not exist.
    #[error("Wallpaper dir {0:?} does not exist.")]
    Parent(PathBuf),

    /// Error resulting from a generic type conversion failure.
    #[error("{0}")]
    TryInto(String),

    /// Error when a system binary or resource cannot be found.
    #[error("Unable to find '{0}'!")]
    UnableToFind(String),

    /// Error for unexpected or unrecognized command-line arguments.
    #[error(
        "{e}: unexpected argument '{a}' found.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        a = arg.yellow(),
        h = "--help".green(),
    )]
    UnexpectedArg { arg: String },
}

// Implement methods on the WallSwitchError enum
impl WallSwitchError {
    /// Extracts the file path associated with an image decoding/opening failure.
    pub fn get_corrupt_path(&self) -> Option<PathBuf> {
        match self {
            Self::CorruptImage { path, .. } => Some(path.clone()),
            _ => None,
        }
    }
}

// Implements the From trait to map standard conversion errors into WallSwitchError.
// Example: let monitor: u8 = value.try_into().map_err(WallSwitchError::from)?;
impl From<TryFromIntError> for WallSwitchError {
    fn from(err: TryFromIntError) -> Self {
        Self::TryInto(err.to_string())
    }
}

/// Dimension Error enum definition
#[derive(Error, Debug)]
pub enum DimensionError {
    /// Error when the parsed dimension value does not meet size constraints.
    #[error(
        "{error}: invalid dimension '{dimension}'.\n\
        {log_min}{log_max}\
        Disregard the path: '{path}'\n",
        error = "Error".red().bold(),
        dimension = .dimension.yellow(),
        log_min = .log_min,
        log_max = .log_max,
        path = .path.display().yellow(),
    )]
    DimensionFormatError {
        dimension: Dimension,
        log_min: String,
        log_max: String,
        path: PathBuf,
    },

    /// Error when the structural format of the dimension is invalid (expected width x height).
    #[error("Invalid dimension format: expected two numbers (width x height)")]
    InvalidFormat,

    /// Error when the dimension data is missing or unreadable for a specific path.
    #[error("Missing or unreadable image dimensions for path: {path}")]
    MissingDimension { path: PathBuf },

    /// Error when image decoding fails while reading dimensions.
    #[error("Failed to read image dimensions for path: {path}. Details: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: ImageError,
    },

    /// Error when a dimension component resolves to zero.
    #[error("Zero is not a valid dimension component")]
    ZeroDimension,
}

#[cfg(test)]
mod error_tests {
    use crate::{Colors, WallSwitchError};
    use std::path::PathBuf;

    #[test]
    /// Can be executed with: `cargo test -- --show-output test_error_display`
    fn test_error_display() {
        let path = PathBuf::from("/tmp");
        let text = format!("Disregard the path: '{}'.", path.display().yellow());
        println!("text: {text}");

        assert_eq!(
            WallSwitchError::InsufficientNumber.to_string(),
            "Insufficient number of valid images!"
        );

        assert_eq!(WallSwitchError::DisregardPath(path).to_string(), text);
    }
}
