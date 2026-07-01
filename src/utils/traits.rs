use crate::{WallSwitchError, WallSwitchResult};
use std::{fmt::Debug, fs, path::Path, thread};

/// An extension trait to perform filesystem-safe, atomic file writes.
pub trait AtomicWriteExt {
    /// Writes content to a temporary file via a closure, then atomically renames
    /// it to the destination path.
    ///
    /// The temporary file is guaranteed to be created in the same directory as
    /// the target path, satisfying POSIX requirements for atomic renames.
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchError::IOError`] if directory creation, the write closure,
    /// or the final rename swap fails.
    fn atomic_write<F>(&self, write_fn: F) -> WallSwitchResult<()>
    where
        F: FnOnce(&Path) -> WallSwitchResult<()>;
}

impl AtomicWriteExt for Path {
    fn atomic_write<F>(&self, write_fn: F) -> WallSwitchResult<()>
    where
        F: FnOnce(&Path) -> WallSwitchResult<()>,
    {
        // 1. Generate a temp file name in the same parent directory
        let temp_ext = format!("tmp-{}", std::process::id());
        let temp_path = self.with_extension(temp_ext);

        // Ensure the parent directory tree exists
        if let Some(parent) = self.parent() {
            fs::create_dir_all(parent).map_err(|e| WallSwitchError::IOError {
                path: parent.to_path_buf(),
                io_error: e,
            })?;
        }

        // 2. Execute the user-provided write closure on the temporary path
        let write_res = write_fn(&temp_path);

        if let Err(err) = write_res {
            // Attempt clean up of the temporary file if writing failed
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }

        // 3. Perform the atomic rename swap
        fs::rename(&temp_path, self).map_err(|io_error| {
            // Clean up the temporary file if the atomic swap fails
            let _ = fs::remove_file(&temp_path);
            WallSwitchError::IOError {
                path: self.to_path_buf(),
                io_error,
            }
        })?;

        Ok(())
    }
}

/// Trait to provide hardware-aware concurrency helpers for collections.
pub trait ConcurrencyExt {
    /// Gets the number of available logical CPU cores.
    /// Defaults to 4 if the OS query fails to ensure a safe baseline.
    fn get_optimal_cores(&self) -> usize {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }

    /// Calculates a balanced chunk size for partitioning work across CPU cores.
    /// Ensures at least 1 item per chunk.
    fn get_chunk_size(&self, total_len: usize) -> usize {
        (total_len / self.get_optimal_cores()).max(1)
    }
}

// Blanket implementation for any slice.
impl<T> ConcurrencyExt for [T] {}

/// Extension trait to calculate the number of digits in an integer.
pub trait DigitWidth {
    /// Returns the number of digits required to display the number.
    /// Example: 0 -> 1, 9 -> 1, 10 -> 2, 100 -> 3.
    fn digit_width(&self) -> usize;
}

impl DigitWidth for usize {
    fn digit_width(&self) -> usize {
        // ilog10(0) is undefined, so checked_ilog10 returns None for 0.
        // We map None to 0 and then add 1 to handle both 0 and the log result correctly.
        self.checked_ilog10().map_or(1, |n| (n + 1) as usize)
    }
}

impl DigitWidth for u64 {
    fn digit_width(&self) -> usize {
        // ilog10(0) is undefined, so checked_ilog10 returns None for 0.
        // We map None to 0 and then add 1 to handle both 0 and the log result correctly.
        self.checked_ilog10().map_or(1, |n| (n + 1) as usize)
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
