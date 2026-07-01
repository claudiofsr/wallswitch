use crate::{
    Colors, Config, DigitWidth, Dimension, DimensionError, WallSwitchResult,
    compute_hashes_parallel, probe_image_dimension,
};
use std::{fmt, path::PathBuf};
use thiserror::Error;

// ==============================================================================
// DOMAIN ENTITIES: Diagnostics & Validation Outcomes
// ==============================================================================

pub type FileValidationResult<T> = Result<T, FileValidationError>;

/// Represents the explicit reasons why a candidate wallpaper file is invalid.
#[derive(Debug, Error)]
pub enum FileValidationError {
    #[error(
        "{error}: Invalid file name --> Disregard the path: '{path}'.",
        error = "Error".red().bold(),
        path = path.display().to_string().yellow()
    )]
    InvalidName { path: PathBuf },

    #[error(
        "{error}: invalid file size '{actual}' bytes. The condition ({min} <= {actual} <= {max}) is false.",
        error = "Error".red().bold(),
        actual = actual_size.to_string().yellow(),
        min = min_size.to_string().green(),
        max = max_size.to_string().green()
    )]
    InvalidSize {
        min_size: u64,
        actual_size: u64,
        max_size: u64,
    },

    /// Error when image dimensions are invalid.
    #[error("{0}")]
    InvalidDimension(#[from] DimensionError),
}

// ==============================================================================
// DOMAIN ENTITY: Pure Data Model
// ==============================================================================

/// Image information representing a wallpaper candidate.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FileInfo {
    /// File number (index + 1) used for display indexing
    pub number: usize,
    /// Total number of files in the current operation
    pub total: usize,
    /// Dimension: width x height of an image.
    pub dimension: Option<Dimension>,
    /// Evaluated dynamically against the current Config.
    pub is_valid: Option<bool>,
    /// The size of the file, in bytes
    pub size: u64,
    /// Unix Timestamp of the last modification (mtime)
    pub mtime: u64,
    /// BLAKE3 Hash of the file contents for visual deduplication
    pub hash: String,
    /// The physical file path of the image
    pub path: PathBuf,
}

impl FileInfo {
    /// Returns true if the given pattern matches a sub-slice of this path.
    pub fn path_contains(&self, string: &str) -> bool {
        self.path.to_str().is_some_and(|p| p.contains(string))
    }

    // --------------------------------------------------------------------------
    // PURE VALIDATION BUSINESS LOGIC (Zero Side Effects, Direct Testing-Friendly)
    // --------------------------------------------------------------------------

    /// Validates all parameters of this file against the system configuration.
    pub fn validate(&self, config: &Config) -> FileValidationResult<()> {
        self.check_name(config)?;
        self.check_size(config)?;
        self.check_dimension(config)?;
        Ok(())
    }

    /// Evaluates if the filename conflicts with the system destination file.
    pub fn check_name(&self, config: &Config) -> FileValidationResult<()> {
        if self.path.file_name() != config.wallpaper.file_name() {
            Ok(())
        } else {
            Err(FileValidationError::InvalidName {
                path: self.path.clone(),
            })
        }
    }

    /// Evaluates if the size boundaries match configuration requirements.
    pub fn check_size(&self, config: &Config) -> FileValidationResult<()> {
        if self.size >= config.min_size && self.size <= config.max_size {
            Ok(())
        } else {
            Err(FileValidationError::InvalidSize {
                min_size: config.min_size,
                actual_size: self.size,
                max_size: config.max_size,
            })
        }
    }

    /// Evaluates image boundaries against structural limits.
    pub fn check_dimension(&self, config: &Config) -> Result<(), DimensionError> {
        let dim = self
            .dimension
            .as_ref()
            .ok_or_else(|| DimensionError::MissingDimension {
                path: self.path.clone(),
            })?;

        if !dim.is_valid(config) {
            return Err(DimensionError::DimensionFormatError {
                dimension: dim.clone(),
                log_min: dim.get_log_min(config),
                log_max: dim.get_log_max(config),
                path: self.path.clone(),
            });
        }

        Ok(())
    }

    // --------------------------------------------------------------------------
    // IMPURE INFRASTRUCTURE FACADES (Encapsulated System Boundaries)
    // --------------------------------------------------------------------------

    /// Probes the image file to retrieve and update its spatial dimensions.
    pub fn update_dimension(&mut self) -> WallSwitchResult<()> {
        self.dimension = Some(probe_image_dimension(&self.path)?);
        Ok(())
    }
}

// ==============================================================================
// DOMAIN LOGIC: Slice Extensions
// ==============================================================================

pub trait FileInfoExt {
    fn get_width_min(&self) -> Option<u64>;
    fn get_max_size(&self) -> Option<u64>;
    fn get_max_number(&self) -> Option<usize>;
    fn get_max_dimension(&self) -> Option<u64>;
    fn update_number(&mut self);
    fn update_hash(&mut self) -> WallSwitchResult<()>;
}

impl FileInfoExt for [FileInfo] {
    fn get_width_min(&self) -> Option<u64> {
        self.iter()
            .filter_map(|f| f.dimension.as_ref().map(|d| d.width))
            .min()
    }

    fn get_max_size(&self) -> Option<u64> {
        self.iter().map(|f| f.size).max()
    }

    fn get_max_number(&self) -> Option<usize> {
        self.iter().map(|f| f.number).max()
    }

    fn get_max_dimension(&self) -> Option<u64> {
        self.iter()
            .filter_map(|f| f.dimension.as_ref().map(|d| d.maximum()))
            .max()
    }

    fn update_number(&mut self) {
        let total = self.len();
        self.iter_mut().enumerate().for_each(|(index, file)| {
            file.number = index + 1;
            file.total = total;
        });
    }

    fn update_hash(&mut self) -> WallSwitchResult<()> {
        compute_hashes_parallel(self);
        Ok(())
    }
}

// ==============================================================================
// PRESENTATION FORMATTERS
// ==============================================================================

pub struct SliceDisplay<'a>(pub &'a [FileInfo]);

impl fmt::Display for SliceDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let digits_n = self.0.get_max_number().map(|n| n.digit_width());
        let digits_s = self.0.get_max_size().map(|s| s.digit_width());
        let digits_d = self.0.get_max_dimension().map(|d| d.digit_width());

        if let (Some(num_digits_number), Some(num_digits_size)) = (digits_n, digits_s) {
            let d_padding = digits_d.unwrap_or(4);

            for file in self.0 {
                let dim_str = match &file.dimension {
                    Some(dim) => format!(
                        "Dimension {{ width: {width:>d$}, height: {height:>d$} }}",
                        width = dim.width,
                        height = dim.height,
                        d = d_padding,
                    ),
                    None => format!(
                        "Dimension {{ {:>width$} }}",
                        "Pending probe",
                        width = d_padding * 2 + 13
                    ),
                };

                writeln!(
                    f,
                    "images[{number:0n$}/{t}]: {dim_str}, size: {size:>s$}, path: {p:?}",
                    number = file.number,
                    n = num_digits_number,
                    t = file.total,
                    size = file.size,
                    s = num_digits_size,
                    p = file.path,
                )?;
            }
        } else {
            return Err(std::fmt::Error);
        }

        Ok(())
    }
}
