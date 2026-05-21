use crate::{Colors, Config, DigitWidth, DimensionError, WallSwitchError, WallSwitchResult};
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
        let numbers: Vec<u64> = split_str(string)?;
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
        let num = self.maximum().digit_width();
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
        let num = max.digit_width();

        write!(
            f,
            "Dimension {{ height: {height:>num$}, width: {width:>num$} }}",
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
    fn split_str_sample_1() -> WallSwitchResult<()> {
        let string = " 123 x 4567 ";
        let numbers = split_str(string)?;

        assert_eq!(numbers[0], 123);
        assert_eq!(numbers[1], 4567);
        Ok(())
    }

    #[test]
    /// Verifies that split_str returns the correct error variant when
    /// provided with a malformed string (e.g., missing width).
    ///
    /// Ref: >https://doc.rust-lang.org/rust-by-example/error.html>
    fn split_str_sample_2() {
        let string = "x4567";

        // 1. Execute the function and capture the error
        let result = split_str(string);
        dbg!(&result);

        // 2. Ensure the result is indeed an error before proceeding
        assert!(
            result.is_err(),
            "The result should be an Err variant for input: {}",
            string
        );

        // 3. Extract the error safely.
        // In tests, .unwrap_err() is acceptable, but let's be more descriptive.
        let err = result.expect_err("Failure expected for empty width component");

        // 4. Use matches! macro to verify the specific Error Hierarchy
        assert!(
            matches!(
                &err,
                WallSwitchError::InvalidDimension(DimensionError::InvalidParse(s, _)) if s == string
            ),
            "Expected InvalidDimension(InvalidParse) for '{}', but got: {:?}",
            string,
            err
        );

        // 5. Verify the end-user error message (Display trait)
        let expected_msg = format!(
            "Invalid dimension format '{string}': failed to parse integer - cannot parse integer from empty string"
        );

        assert_eq!(err.to_string(), expected_msg);
    }

    #[test]
    /// Verifies that `split_str` correctly identifies and handles non-numeric
    /// characters within the dimension components.
    fn split_str_sample_3() {
        let string = "12ab3x4567";
        let parse_error_reason = "invalid digit found in string";

        // 1. Execute the split operation
        let result = split_str(string);

        // 2. Ensure the result is an error.
        // Using .expect_err() provides a clear failure message if the code unexpectedly succeeds.
        let err = result.expect_err("Parsing should fail when non-numeric characters are present");

        // 3. Use matches! macro to verify the error hierarchy and the inner data.
        // We check if it's an InvalidDimension wrapping an InvalidParse,
        // and validate both the failing string and the underlying ParseIntError message.
        assert!(
            matches!(
                &err,
                WallSwitchError::InvalidDimension(DimensionError::InvalidParse(s, parse_err))
                if s == string && parse_err.to_string() == parse_error_reason
            ),
            "Expected InvalidDimension(InvalidParse) for input '{}' with reason '{}', but got: {:?}",
            string,
            parse_error_reason,
            err
        );

        // 4. Validate the end-user error message generated by the Display trait.
        // This ensures the error reporting is clear and matches expectations.
        let expected_msg = format!(
            "Invalid dimension format '{string}': failed to parse integer - {parse_error_reason}"
        );

        assert!(
            err.to_string().contains(&expected_msg),
            "The error message should contain the formatted reason.\nExpected: {}\nActual: {}",
            expected_msg,
            err
        );
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
