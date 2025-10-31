use crate::{
    Colors, Config, Countable, DimensionError, ResultExt, WallSwitchError, WallSwitchResult,
};
use serde::{Deserialize, Serialize};
use std::{fmt, num::ParseIntError};

/// Dimension - width and length - of an image.
///
/// Image Size, Attribute, properties.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Dimension {
    /// width of an image
    pub width: u64,
    /// length of an image
    pub height: u64,
}

impl Default for Dimension {
    /// Default 4K UHD Resolution
    fn default() -> Self {
        Dimension {
            width: 3840,
            height: 2160,
        }
    }
}

impl Dimension {
    /// Get an instance of Dimension by specifying concrete values ​​for each of the fields.
    pub fn new(string: &str) -> WallSwitchResult<Dimension> {
        let numbers: Vec<u64> = split_str(string).unwrap_result();
        let (width, height) = (numbers[0], numbers[1]);
        Ok(Dimension { width, height })
    }

    /// Get the minimum value between height and width.
    pub fn minimum(&self) -> u64 {
        self.height.min(self.width)
    }

    /// Get the maximum value between height and width.
    pub fn maximum(&self) -> u64 {
        self.height.max(self.width)
    }

    /// Check if the minimum and maximum value are valid.
    pub fn is_valid(&self, config: &Config) -> bool {
        config.in_range(self.minimum()) && config.in_range(self.maximum())
    }

    /// Get error messages related to the minimum value.
    pub fn get_log_min(&self, config: &Config) -> String {
        // Number of digits of maximum value
        let num = self.maximum().count_chars();
        let min = self.minimum();
        let min = format!("{min:>num$}");
        if !config.in_range(self.minimum()) {
            format!(
                "Minimum dimension: {min}. The condition ({config_min} <= {min} <= {config_max}) is false.\n",
                min = min.yellow(),
                config_min = config.min_dimension.green(),
                config_max = config.max_dimension.green(),
            )
        } else {
            "".to_string()
        }
    }

    /// Get error messages related to the maximum value.
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let height = self.height;
        let width = self.width;

        let max = height.max(width);
        let n = max.count_chars();

        write!(
            f,
            "Dimension {{ height: {height:>n$}, width: {width:>n$} }}",
        )
    }
}

/**
Split string into two numbers

Example:
```
use wallswitch::{split_str, WallSwitchResult};

fn main() -> WallSwitchResult<()> {
    let string1: &str = "123x4567";
    let string2: &str = " 123 x 4567 \n";

    let integers1: Vec<u64> = split_str(string1)?;
    let integers2: Vec<u64> = split_str(string2)?;

    assert_eq!(integers1, [123, 4567]);
    assert_eq!(integers1, integers2);

    Ok(())
}
```
*/
pub fn split_str(string: &str) -> WallSwitchResult<Vec<u64>> {
    let numbers: Vec<u64> = string
        .split('x')
        .map(|s| s.trim().parse::<u64>())
        .collect::<Result<Vec<u64>, ParseIntError>>()
        .map_err(|parse_error| {
            // Corrected: Directly return DimensionError, which WallSwitchError can convert from
            WallSwitchError::InvalidDimension(DimensionError::InvalidParse(
                string.to_string(),
                parse_error,
            ))
        })?;

    if numbers.len() != 2 {
        // Corrected: Directly return DimensionError, which WallSwitchError can convert from
        return Err(WallSwitchError::InvalidDimension(
            DimensionError::InvalidFormat,
        ));
    }

    if numbers.contains(&0) {
        // Corrected: Directly return DimensionError, which WallSwitchError can convert from
        return Err(WallSwitchError::InvalidDimension(
            DimensionError::ZeroDimension,
        ));
    }

    Ok(numbers)
}

#[cfg(test)]
mod test_dimension {
    use super::*;

    #[test]
    /// `cargo test -- --show-output split_str_sample_1`
    fn split_str_sample_1() {
        let string = " 123 x 4567 ";

        let result = split_str(string);

        match result {
            Ok(numbers) => {
                assert_eq!(numbers[0], 123);
                assert_eq!(numbers[1], 4567);
            }
            Err(err) => panic!("Unexpected error: {err}"),
        }
    }

    #[test]
    /// `cargo test -- --show-output split_str_sample_2`
    ///
    /// <https://doc.rust-lang.org/rust-by-example/error.html>
    ///
    /// <https://world.hey.com/ksiva/easy-error-handling-in-rust-19555479>
    fn split_str_sample_2() {
        let string = "x4567";

        let result: WallSwitchResult<Vec<u64>> = split_str(string);
        dbg!(&result);

        let result_to_string: Result<Vec<u64>, String> =
            split_str(string).map_err(|e| e.to_string());
        dbg!(&result_to_string);

        let opt_error: String = result_to_string.unwrap_err();

        let expected_exact_error_message = format!(
            "Invalid dimension format '{string}': failed to parse integer - cannot parse integer from empty string"
        );

        // Assert that the result is an error
        assert!(result.is_err());

        // Pattern match to check the specific error variant
        match result.unwrap_err() {
            WallSwitchError::InvalidDimension(DimensionError::InvalidParse(err_string, _)) => {
                assert_eq!(err_string, string);
            }
            other_error => panic!(
                "Expected InvalidDimension(InvalidParse), but got: {:?}",
                other_error
            ),
        }

        // Compare the full error string exactly
        assert_eq!(opt_error, expected_exact_error_message);
    }

    #[test]
    /// `cargo test -- --show-output split_str_sample_3`
    fn split_str_sample_3() {
        let string = "12ab3x4567";

        let result: WallSwitchResult<Vec<u64>> = split_str(string);
        dbg!(&result);

        // Using `map_err` to convert to a String for comparison, which is fine
        let result_to_string: Result<Vec<u64>, String> =
            split_str(string).map_err(|e| e.to_string());
        dbg!(&result_to_string);

        let opt_error: String = result_to_string.unwrap_err(); // .to_string() is already done in map_err
        let parse_error_reason = "invalid digit found in string"; // The actual ParseIntError message
        let expected_output_part = format!(
            "Invalid dimension format '{string}': failed to parse integer - {parse_error_reason}"
        );

        // Assert that the result is an error
        assert!(result.is_err());

        // Pattern match to check the specific error variant
        // We expect WallSwitchError::InvalidDimension wrapping DimensionError::InvalidParse
        match result.unwrap_err() {
            WallSwitchError::InvalidDimension(DimensionError::InvalidParse(
                err_string,
                parse_err,
            )) => {
                // Check if the contained string matches expectations
                assert_eq!(err_string, string);
                // Check the underlying ParseIntError's message
                assert_eq!(parse_err.to_string(), parse_error_reason);
            }
            other_error => panic!(
                "Expected InvalidDimension(InvalidParse), but got: {:?}",
                other_error
            ),
        }

        // Compare the full error string from .to_string()
        // Using contains as the error message might include more details than just `output`
        assert!(opt_error.contains(&expected_output_part));
    }

    #[test]
    /// `cargo test -- --show-output split_str_sample_4`
    fn split_str_sample_4() {
        let string = "57x124x89";

        let result = split_str(string);
        dbg!(&result);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid dimension format: expected two numbers (width x height)"
        );
    }

    #[test]
    /// `cargo test -- --show-output split_str_sample_5`
    fn split_str_sample_5() {
        let string = "57x0";

        let result = split_str(string);
        dbg!(&result);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Zero is not a valid dimension component"
        );
    }
}
