use crate::{Colors, Config, DigitWidth};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents the physical dimensions (width and height) of an image.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Dimension {
    /// The width of the image in pixels.
    pub width: u64,
    /// The height of the image in pixels.
    pub height: u64,
}

impl Default for Dimension {
    /// Returns a default resolution of 4K UHD (3840 x 2160).
    fn default() -> Self {
        Dimension {
            width: 3840,
            height: 2160,
        }
    }
}

impl Dimension {
    /// Returns the smaller value between height and width.
    pub fn minimum(&self) -> u64 {
        self.height.min(self.width)
    }

    /// Returns the larger value between height and width.
    pub fn maximum(&self) -> u64 {
        self.height.max(self.width)
    }

    /// Checks if both the minimum and maximum dimensions are within the allowed range defined by the configuration.
    pub fn is_valid(&self, config: &Config) -> bool {
        config.in_range(self.minimum()) && config.in_range(self.maximum())
    }

    /// Generates a formatted error message if the minimum dimension is outside the configured range.
    /// Returns an empty string if the value is within range.
    pub fn get_log_min(&self, config: &Config) -> String {
        // Determine the number of digits of the maximum value for alignment purposes.
        let num = self.maximum().digit_width();
        let min = self.minimum();
        let min_str = format!("{min:>num$}");

        if !config.in_range(self.minimum()) {
            format!(
                "Minimum dimension: {min}. The condition ({config_min} <= {min} <= {config_max}) is false.\n",
                min = min_str.yellow(),
                config_min = config.min_dimension.green(),
                config_max = config.max_dimension.green(),
            )
        } else {
            "".to_string()
        }
    }

    /// Generates a formatted error message if the maximum dimension is outside the configured range.
    /// Returns an empty string if the value is within range.
    pub fn get_log_max(&self, config: &Config) -> String {
        if !config.in_range(self.maximum()) {
            format!(
                "Maximum dimension: {max}. The condition ({config_min} <= {max} <= {config_max}) is false.\n",
                max = self.maximum().yellow(),
                config_min = config.min_dimension.green(),
                config_max = config.max_dimension.green(),
            )
        } else {
            "".to_string()
        }
    }
}

impl fmt::Display for Dimension {
    /// Formats the dimensions with aligned width based on the maximum digit length.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height;
        let width = self.width;

        let max = height.max(width);
        let num = max.digit_width();

        write!(
            f,
            "Dimension {{ height: {height:>num$}, width: {width:>num$} }}",
        )
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// Run tests with:
///
/// cargo test -- --show-output tests_dimension
#[cfg(test)]
mod tests_dimension {
    use super::*;

    #[test]
    fn test_default_dimension() {
        let default_dim = Dimension::default();
        assert_eq!(default_dim.width, 3840);
        assert_eq!(default_dim.height, 2160);
    }

    #[test]
    fn test_minimum_and_maximum() {
        // Case: landscape orientation
        let landscape = Dimension {
            width: 1920,
            height: 1080,
        };
        assert_eq!(landscape.minimum(), 1080);
        assert_eq!(landscape.maximum(), 1920);

        // Case: portrait orientation
        let portrait = Dimension {
            width: 1080,
            height: 1920,
        };
        assert_eq!(portrait.minimum(), 1080);
        assert_eq!(portrait.maximum(), 1920);

        // Case: square
        let square = Dimension {
            width: 1000,
            height: 1000,
        };
        assert_eq!(square.minimum(), 1000);
        assert_eq!(square.maximum(), 1000);
    }

    #[test]
    fn test_display_formatting() {
        let dim = Dimension {
            width: 120,
            height: 8,
        };
        let formatted = format!("{}", dim);

        // The maximum value is 120, which has 3 digits.
        // Height (8) should be padded to 3 spaces: "  8".
        // Width (120) should be formatted as: "120".
        assert!(formatted.contains("height:   8"));
        assert!(formatted.contains("width: 120"));
    }

    // Example of testing behavior with a Config instance.
    // Replace this structure initialization with your actual Config creation pattern.
    #[test]
    fn test_validation_and_logs() {
        // Assuming Config has fields or a constructor that can be initialized for testing.
        // For example purposes, we assume standard instantiation:
        let config = Config {
            min_dimension: 100,
            max_dimension: 5000,
            ..Default::default()
        };

        // Case 1: Dimension within valid boundaries
        let valid_dim = Dimension {
            width: 1920,
            height: 1080,
        };
        assert!(valid_dim.is_valid(&config));
        assert_eq!(valid_dim.get_log_min(&config), "");
        assert_eq!(valid_dim.get_log_max(&config), "");

        // Case 2: Dimension below minimum boundary
        let small_dim = Dimension {
            width: 50,
            height: 1080,
        };
        assert!(!small_dim.is_valid(&config));
        assert!(!small_dim.get_log_min(&config).is_empty());

        // Case 3: Dimension above maximum boundary
        let large_dim = Dimension {
            width: 6000,
            height: 1080,
        };
        assert!(!large_dim.is_valid(&config));
        assert!(!large_dim.get_log_max(&config).is_empty());
    }
}
