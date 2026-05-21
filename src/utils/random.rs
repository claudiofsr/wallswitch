use crate::{WallSwitchError, WallSwitchResult};
use std::hash::{BuildHasher, Hasher, RandomState};

/// Trait to extend slices with shuffling capabilities.
pub trait RandomExt {
    /// Shuffle the elements in place using the Fisher-Yates algorithm.
    fn shuffle(&mut self);
}

impl<T> RandomExt for [T] {
    /**
    Shuffle the vector in place with the Fisher-Yates algorithm.

    ```
        use wallswitch::RandomExt;

        let mut strings = vec!["abc", "foo", "bar", "baz", "mm nn", "zzz"];

        strings.shuffle();

        println!("strings: {:?}", strings);

        let mut integers: Vec<u32> = (1..=20).collect();

        integers.shuffle();

        println!("integers: {:?}", integers);
    ```

    <https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle>

    <https://stackoverflow.com/questions/26033976/how-do-i-create-a-vec-from-a-range-and-shuffle-it>

    */
    fn shuffle(&mut self) {
        let n: usize = self.len();
        if n < 2 {
            return;
        }

        for i in 0..(n - 1) {
            // Generate random index j, such that: i <= j < n
            let j = (rand() as usize) % (n - i) + i;
            self.swap(i, j);
        }
    }
}

/// Generate random numbers without external dependencies.
///
/// It utilizes the system's 'RandomState' to provide a
/// seed-like value based on the hasher's internal state.
pub fn rand() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// Generate a random integer value in the given range (min, max) inclusive.
pub fn get_random_integer(min: u64, max: u64) -> u64 {
    if min >= max {
        return min;
    }
    min + rand() % (max - min + 1)
}

/// Generate a random integer value in the given range (min, max) inclusive.
///
/// Returns an error if `min > max`.
pub fn get_random_integer_safe(min: u64, max: u64) -> WallSwitchResult<u64> {
    if min > max {
        Err(WallSwitchError::MinMax { min, max })
    } else {
        Ok(min + rand() % (max - min + 1))
    }
}
