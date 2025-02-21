// use colored::Colorize;
use crate::{Colors, Dimension, Orientation};
use std::{
    error::Error,
    fmt::{self, Debug},
    io,
    num::TryFromIntError,
    path::PathBuf,
};

/// WallSwitch Error enum
///
/// The `WSError` enum defines the error values
///
/// <https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html>
#[derive(Debug)]
pub enum WSError<'a> {
    /// Unable to find
    UnableToFind(&'a str),
    /// Unable to obtain minimum value
    MinValue,
    /// Unable to obtain maximum value
    MaxValue,
    /// Directory path must exist
    Parent(PathBuf),
    /// Insufficient number of valid images
    InsufficientNumber,
    /// Try to performs the conversion
    TryInto(String),
    /// Invalid dimension
    InvalidDimension(DimensionError),
    /// Invalid orientation
    InvalidOrientation,
    /// Writing/Reading error
    IOError(PathBuf, io::Error),
    /// No images found
    NoImages(Vec<PathBuf>),
    /// Insufficient number of image files
    InsufficientImages(Vec<PathBuf>, usize),
    /// Minimum value > Maximum value
    MinMax(u64, u64),
    /// Missing value
    MissingValue(&'a str),
    /// Invalid value
    InvalidValue(&'a str, String),
    /// At least value
    AtLeastValue(&'a str, String, u64),
    /// At most value
    AtMostValue(&'a str, String, u64),
    /// Unexpected argument
    UnexpectedArg(String),
    /// Invalid file size
    InvalidSize(u64, u64, u64),
    /// Disregard the file path
    DisregardPath(PathBuf),
    /// Invalid file name
    InvalidFilename(PathBuf),
}

// Implement Display for WSError
impl fmt::Display for WSError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WSError::UnableToFind(cmd) => write!(f, "Unable to find '{cmd}'!"),
            WSError::MinValue => write!(f, "Unable to obtain minimum value!"),
            WSError::MaxValue => write!(f, "Unable to obtain maximum value!"),
            WSError::Parent(path) => write!(f, "Wallpaper dir {path:?} does not exist."),
            WSError::InsufficientNumber => write!(f, "Insufficient number of valid images!"),
            WSError::TryInto(err) => write!(f, "{err}"),
            WSError::InvalidDimension(dim_error) => write!(f, "{dim_error}"),
            WSError::InvalidOrientation => write!(
                f,
                "invalid orientation.\n\n\
                Valid options: [{h}, {v}]",
                h = Orientation::Horizontal.green(),
                v = Orientation::Vertical.green(),
            ),
            WSError::IOError(path, io_error) => write!(
                f,
                "{e}: Failed to create file {path:?}\n{io_error}",
                e = "Error".red().bold(),
            ),
            WSError::NoImages(paths) => write!(
                f,
                "{e}: no images found in image directories!\n\
                directories: {paths:#?}",
                e = "Error".red().bold(),
            ),
            WSError::InsufficientImages(paths, nfiles) => write!(
                f,
                "{e}: insufficient number of image files!\n\
                Found only {n} image file(s):\n\
                {paths:#?}",
                n = nfiles.yellow(),
                e = "Error".red().bold(),
            ),
            WSError::MinMax(min, max) => write!(
                f,
                "{e}: min ({min}) must be less than or equal to max ({max})\n\
                The condition ({min} <= {max}) is false.",
                e = "Error".red().bold()
            ),
            WSError::MissingValue(arg) => write!(
                f,
                "{e}: missing value for '{a}'.\n\n\
                For more information, try '{h}'.",
                e = "Error".red().bold(),
                a = arg.yellow(),
                h = "--help".green(),
            ),
            WSError::InvalidValue(arg, value) => write!(
                f,
                "{e}: invalid value '{v}' for '{a}'.\n\n\
                For more information, try '{h}'.",
                e = "Error".red().bold(),
                v = value.yellow(),
                a = arg.green(),
                h = "--help".green(),
            ),
            WSError::AtLeastValue(arg, value, num) => write!(
                f,
                "{e}: '{a}' value '{v}' must be at least '{n}'. \
                The condition ({v} >= {n}) is false.\n\n\
                For more information, try '{h}'.",
                e = "Error".red().bold(),
                v = value.yellow(),
                a = arg.yellow(),
                n = num.green(),
                h = "--help".green(),
            ),
            WSError::AtMostValue(arg, value, num) => write!(
                f,
                "{e}: '{a}' value '{v}' must be at most '{n}'. \
                The condition ({v} <= {n}) is false.\n\n\
                For more information, try '{h}'.",
                e = "Error".red().bold(),
                v = value.yellow(),
                a = arg.yellow(),
                n = num.green(),
                h = "--help".green(),
            ),
            WSError::UnexpectedArg(arg) => write!(
                f,
                "{e}: unexpected argument '{a}' found.\n\n\
                For more information, try '{h}'.",
                e = "Error".red().bold(),
                a = arg.yellow(),
                h = "--help".green(),
            ),
            WSError::InvalidSize(min_size, size, max_size) => write!(
                f,
                "{e}: invalid file size '{s}' bytes. \
                The condition ({min} <= {s} <= {max}) is false.",
                e = "Error".red().bold(),
                min = min_size.green(),
                max = max_size.green(),
                s = size.yellow(),
            ),
            WSError::DisregardPath(path) => {
                write!(f, "Disregard the path: '{p}'.", p = path.display().yellow(),)
            }
            WSError::InvalidFilename(path) => write!(
                f,
                "{e}: Invalid file name --> Disregard the path: '{p}'.",
                e = "Error".red().bold(),
                p = path.display().yellow(),
            ),
        }
    }
}

/// If we want to use std::error::Error in main, we need to implement it for WSError
impl Error for WSError<'_> {}

// Implementing the From trait for multiple types that you want to map into WSError.
// https://stackoverflow.com/questions/62238827/less-verbose-type-for-map-err-closure-argument
// let monitor: u8 = value.try_into().map_err(WSError::from)?;

impl From<TryFromIntError> for WSError<'_> {
    fn from(err: TryFromIntError) -> Self {
        Self::TryInto(err.to_string())
    }
}

/// Dimension Error
#[derive(Debug)]
pub struct DimensionError {
    pub dimension: Dimension,
    pub log_min: String,
    pub log_max: String,
    pub path: PathBuf,
}

impl fmt::Display for DimensionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{error}: invalid dimension '{dimension}'.\n\
            {log_min}{log_max}\
            Disregard the path: '{path}'\n",
            error = "Error".red().bold(),
            dimension = self.dimension.yellow(),
            log_min = self.log_min,
            log_max = self.log_max,
            path = self.path.display().yellow(),
        )
    }
}

impl Error for DimensionError {}

#[cfg(test)]
mod error_tests {
    use crate::{Colors, WSError};
    use std::path::PathBuf;

    #[test]
    /// `cargo test -- --show-output test_error_display`
    fn test_error_display() {
        let path = PathBuf::from("/tmp");
        let text = format!("Disregard the path: '{}'.", path.display().yellow());

        assert_eq!(
            WSError::InsufficientNumber.to_string(),
            "Insufficient number of valid images!"
        );

        assert_eq!(WSError::DisregardPath(path).to_string(), text);
    }
}
