use crate::{Colors, Config, Countable, MyResult, ResultExt};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, num::ParseIntError};

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
    pub fn new(string: &str) -> MyResult<Dimension> {
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
use wallswitch::{split_str, MyResult};

fn main() -> MyResult<()> {
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
pub fn split_str(string: &str) -> MyResult<Vec<u64>> {
    let numbers: Vec<u64> = string
        .split('x')
        .map(|s| s.trim().parse::<u64>())
        .collect::<Result<Vec<u64>, ParseIntError>>()
        .map_err(|parse_error| {
            // Add a custom error message
            DimError::InvalidParse(string.to_string(), parse_error)
        })?;

    if numbers.len() != 2 {
        return Err(Box::new(DimError::InvalidFormat));
    }

    if numbers.contains(&0) {
        return Err(Box::new(DimError::ZeroDimension));
    }

    Ok(numbers)
}

#[derive(Debug)]
enum DimError {
    /// Parse InvalidFormat
    InvalidFormat,
    /// Parse ZeroDimension
    ZeroDimension,
    /// Parse InvalidParse
    InvalidParse(String, ParseIntError),
}

impl std::fmt::Display for DimError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DimError::InvalidParse(string, parse_error) => {
                write!(f, "Error: '{string}' split failed!\n{parse_error}",)
            }
            DimError::InvalidFormat => write!(f, "Invalid format: expected 'heightxwidth'."),
            DimError::ZeroDimension => write!(f, "Width or height cannot be zero."),
        }
    }
}

/// If we want to use std::error::Error in main, we need to implement it for DimError
impl Error for DimError {}

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

        let result: MyResult<Vec<u64>> = split_str(string);
        dbg!(&result);

        let result_to_string: Result<Vec<u64>, String> =
            split_str(string).map_err(|e| e.to_string());
        dbg!(&result_to_string);

        let opt_error: String = result_to_string.unwrap_err().to_string();
        let parse_error = "cannot parse integer from empty string";
        let output = format!("Error: '{string}' split failed!\n{parse_error}",);

        let error = split_str(string).unwrap_err();
        assert!(error.is::<DimError>());

        assert!(result.is_err());
        assert_eq!(opt_error, output);
    }

    #[test]
    /// `cargo test -- --show-output split_str_sample_3`
    fn split_str_sample_3() {
        let string = "12ab3x4567";

        let result: MyResult<Vec<u64>> = split_str(string);
        dbg!(&result);

        let result_to_string: Result<Vec<u64>, String> =
            split_str(string).map_err(|e| e.to_string());
        dbg!(&result_to_string);

        let opt_error: String = result_to_string.unwrap_err().to_string();
        let parse_error = "invalid digit found in string";
        let output = format!("Error: '{string}' split failed!\n{parse_error}",);

        let error = split_str(string).unwrap_err();
        assert!(error.is::<DimError>());

        assert!(result.is_err());
        assert_eq!(opt_error, output);
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
            "Invalid format: expected 'heightxwidth'."
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
            "Width or height cannot be zero."
        );
    }
}
