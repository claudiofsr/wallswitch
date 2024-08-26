use std::fmt::{self, Debug, Display};

// https://gist.github.com/abritinthebay/d80eb99b2726c83feb0d97eab95206c4
// https://talyian.github.io/ansicolors/

// Ansi Colors:
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";

pub trait Colors {
    fn bold(&self) -> String;
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn blue(&self) -> String;
}

impl<T> Colors for T
where
    T: Display, // + Deref<Target = str>
{
    fn bold(&self) -> String {
        format!("{BOLD}{self}{RESET}")
    }

    fn red(&self) -> String {
        format!("{RED}{self}{RESET}")
    }

    fn green(&self) -> String {
        format!("{GREEN}{self}{RESET}")
    }

    fn yellow(&self) -> String {
        format!("{YELLOW}{self}{RESET}")
    }

    fn blue(&self) -> String {
        format!("{BLUE}{self}{RESET}")
    }
}

/// Print Extension with Debug
pub trait PrintWithSpaces {
    /// Print Slice `[T]` with spaces
    fn print_with_spaces(&self, spaces: &str);
}

impl<T> PrintWithSpaces for [T]
where
    T: Debug,
{
    fn print_with_spaces(&self, spaces: &str) {
        for item in self {
            println!("{spaces}{item:?}");
        }
    }
}

/**
Find the maximum value of `Vec<f64>`.

Example:
```
    use wallswitch::FloatIterExt;

    let vector: Vec<f64> = vec![4.2, -3.7, 8.1, 0.9];
    let max = vector
        .iter()
        .cloned()
        .float_max();

    assert_eq!(max, 8.1);
```
<https://www.reddit.com/r/rust/comments/3fg0xr/how_do_i_find_the_max_value_in_a_vecf64/>
*/
pub trait FloatIterExt {
    fn float_min(&mut self) -> f64;
    fn float_max(&mut self) -> f64;
}

impl<T> FloatIterExt for T
where
    T: Iterator<Item = f64>,
{
    fn float_max(&mut self) -> f64 {
        self.fold(f64::NAN, f64::max)
    }

    fn float_min(&mut self) -> f64 {
        self.fold(f64::NAN, f64::min)
    }
}

/**
Find the maximum value of `Vec<u32>`.

Example:
```
    use  wallswitch::IntegerIterExt;

    let vector: Vec<u32> = vec![4, 3, 2, 8];
    let min = vector
        .iter()
        .cloned()
        .integer_min();

    assert_eq!(min, 2);
```
*/
pub trait IntegerIterExt {
    fn integer_min(&mut self) -> u32;
    fn integer_max(&mut self) -> u32;
}

impl<T> IntegerIterExt for T
where
    T: Iterator<Item = u32>,
{
    fn integer_max(&mut self) -> u32 {
        self.fold(u32::MIN, u32::max)
    }

    fn integer_min(&mut self) -> u32 {
        self.fold(u32::MAX, u32::min)
    }
}

/// Trait for counting the number of chars
pub trait Countable {
    fn count_chars(&self) -> usize;
}

impl<T> Countable for T
where
    T: ToString,
{
    fn count_chars(&self) -> usize {
        self.to_string().chars().count()
    }
}

/// Result Extension
pub trait ResultExt<T> {
    /// If OK, unwrap Result<T, Error> to the value T.
    ///
    /// If Error, terminate the current process with error messages.
    fn unwrap_result(self) -> T;
}

impl<T, E: fmt::Display> ResultExt<T> for Result<T, E> {
    fn unwrap_result(self) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
}

/// u8 Extension
pub trait U8Extension {
    /// Convert u8 to usize
    fn to_usize(self) -> usize;
    /// Convert u8 to u64
    fn to_u64(self) -> u64;
}

impl U8Extension for u8 {
    fn to_usize(self) -> usize {
        Into::<usize>::into(self)
    }

    fn to_u64(self) -> u64 {
        Into::<u64>::into(self)
    }
}
