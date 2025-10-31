use crate::{Colors, Dimension, Orientation};
use std::{
    io,
    num::{ParseIntError, TryFromIntError},
    path::PathBuf,
    string::FromUtf8Error,
};
use thiserror::Error;

/**
Result type to simplify function signatures.

This is a custom result type that uses our custom `WallSwitchError` for the error type.

Functions can return `WallSwitchResult<T>` and then use `?` to automatically propagate errors.
*/
pub type WallSwitchResult<T> = Result<T, WallSwitchError>;

/// WallSwitch Error enum
///
/// The `WallSwitchError` enum defines the error values
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

    /// Error when an image path should be disregarded.
    #[error("Disregard the path: '{p}'.", p = .0.display().yellow(),)]
    DisregardPath(PathBuf),

    /// Error when failing to convert byte output to a UTF-8 string.
    #[error("Failed to convert command output to UTF-8: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    /// Standard I/O error wrapper.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Error when an image's dimensions are invalid.
    #[error("{0}")]
    InvalidDimension(#[from] DimensionError),

    /// Error for file paths that have invalid filenames.
    #[error(
        "{e}: Invalid file name --> Disregard the path: '{p}'.",
        e = "Error".red().bold(),
        p = .0.display().yellow(),
    )]
    InvalidFilename(PathBuf),

    /// Error for image file sizes that are outside an allowed range.
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

    /// Error for command-line arguments that have invalid values.
    #[error(
        "{e}: invalid value '{v}' for '{a}'.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        v = value.yellow(),
        a = arg.green(),
        h = "--help".green(),
    )]
    InvalidValue { arg: String, value: String },

    /// Error for I/O operations with an associated file path.
    #[error("{e}: Failed to create file {path:?}\n{io_error}", e = "Error".red().bold())]
    IOError {
        path: PathBuf,
        #[source]
        io_error: io::Error,
    },

    /// Error when a JSON serialization or deserialization operation fails.
    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error when obtaining the maximum valid value for a parameter.
    #[error("Unable to obtain maximum value!")]
    MaxValue,

    /// Error when obtaining the minimum valid value for a parameter.
    #[error("Unable to obtain minimum value!")]
    MinValue,

    /// Error for command-line arguments that are missing required values.
    #[error(
        "{e}: missing value for '{a}'.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        a = arg.yellow(),
        h = "--help".green(),
    )]
    MissingValue { arg: String },

    /// Error for minimum value being greater than maximum value.
    #[error(
        "{e}: min ({min}) must be less than or equal to max ({max})\n\
        The condition ({min} <= {max}) is false.",
        e = "Error".red().bold()
    )]
    MinMax { min: u64, max: u64 },

    /// Error when no valid images are found in specified directories.
    #[error(
        "{e}: no images found in image directories!\n\
        directories: {paths:#?}",
        e = "Error".red().bold(),
    )]
    NoImages { paths: Vec<PathBuf> },

    /// Error for invalid image orientation (e.g., neither horizontal nor vertical).
    #[error(
        "invalid orientation.\n\n\
        Valid options: [{h}, {v}]",
        h = Orientation::Horizontal.green(),
        v = Orientation::Vertical.green(),
    )]
    InvalidOrientation,

    /// Error for an insufficient number of image files found.
    #[error(
        "{e}: insufficient number of image files!\n\
        Found only {n} image file(s):\n\
        {paths:#?}",
        n = nfiles.yellow(),
        e = "Error".red().bold(),
    )]
    InsufficientImages { paths: Vec<PathBuf>, nfiles: usize },

    /// Error when an insufficient number of valid images are present.
    #[error("Insufficient number of valid images!")]
    InsufficientNumber,

    /// Error when a directory path does not exist.
    #[error("Wallpaper dir {0:?} does not exist.")]
    Parent(PathBuf),

    /// Error from a generic conversion attempt.
    #[error("{0}")]
    TryInto(String),

    /// Error when a binary or resource cannot be found on the system.
    #[error("Unable to find '{0}'!")]
    UnableToFind(String),

    /// Error for unexpected command-line arguments.
    #[error(
        "{e}: unexpected argument '{a}' found.\n\n\
        For more information, try '{h}'.",
        e = "Error".red().bold(),
        a = arg.yellow(),
        h = "--help".green(),
    )]
    UnexpectedArg { arg: String },
}

// Implementing the From trait for multiple types that you want to map into WallSwitchError.
// https://stackoverflow.com/questions/62238827/less-verbose-type-for-map-err-closure-argument
// let monitor: u8 = value.try_into().map_err(WallSwitchError::from)?;

impl From<TryFromIntError> for WallSwitchError {
    fn from(err: TryFromIntError) -> Self {
        Self::TryInto(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum DimensionError {
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

    #[error("Invalid dimension format '{0}': failed to parse integer - {1}")]
    InvalidParse(String, #[source] ParseIntError),

    #[error("Invalid dimension format: expected two numbers (width x height)")]
    InvalidFormat,

    #[error("Zero is not a valid dimension component")]
    ZeroDimension,
}

#[cfg(test)]
mod error_tests {
    use crate::{Colors, WallSwitchError};
    use std::path::PathBuf;

    #[test]
    /// `cargo test -- --show-output test_error_display`
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
