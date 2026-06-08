use crate::{WallSwitchError, WallSwitchResult};
use std::hash::{BuildHasher, Hasher, RandomState};

/// Trait to extend slices with shuffling and random selection capabilities.
pub trait RandomExt {
    /// The type of elements in the slice.
    type Item;

    /// Shuffles the elements of the slice in place using the Fisher-Yates algorithm.
    ///
    /// This operation runs in $O(N)$ time complexity and modifies the collection
    /// directly without allocating extra memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use wallswitch::RandomExt;
    ///
    /// let mut strings = vec!["abc", "foo", "bar", "baz", "mm nn", "zzz"];
    /// strings.shuffle();
    ///
    /// let mut integers: Vec<u32> = (1..=20).collect();
    /// integers.shuffle();
    /// ```
    ///
    /// See: <https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle>
    fn shuffle(&mut self);

    /// Chooses a random reference to an element from the slice.
    ///
    /// Returns `None` if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use wallswitch::RandomExt;
    ///
    /// let items = vec![10, 20, 30];
    /// let chosen = items.choose();
    /// assert!(chosen.is_some());
    /// assert!(items.contains(chosen.unwrap()));
    /// ```
    fn choose(&self) -> Option<&Self::Item>;

    /// Selects a random copy of an element from the slice.
    ///
    /// This is a convenience method that combines choosing an element and copying it.
    /// It returns a [`WallSwitchResult`] to support clean error propagation using
    /// the `?` operator.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use wallswitch::{RandomExt, WallSwitchResult};
    ///
    /// # fn main() -> WallSwitchResult<()> {
    /// let items = vec![10, 20, 30];
    /// let chosen = items.get_random_sample()?;
    /// assert!(items.contains(&chosen));
    /// # Ok(())
    /// # }
    /// ```
    fn get_random_sample(&self) -> WallSwitchResult<Self::Item>
    where
        Self::Item: Copy;

    /// Selects a random cloned copy of an element from the slice.
    ///
    /// This is a convenience method that combines choosing an element and cloning it,
    /// supporting types that do not implement the `Copy` trait.
    ///
    /// # Errors
    ///
    /// Returns [`WallSwitchError::EmptySlice`](crate::WallSwitchError::EmptySlice)
    /// if the slice is empty.
    fn get_random_sample_cloned(&self) -> WallSwitchResult<Self::Item>
    where
        Self::Item: Clone;
}

impl<T> RandomExt for [T] {
    type Item = T;

    fn shuffle(&mut self) {
        let n: usize = self.len();
        if n < 2 {
            return;
        }

        for i in 0..(n - 1) {
            // Generate random index j, such that: i <= j < n
            let j = rand::<usize>() % (n - i) + i;
            self.swap(i, j);
        }
    }

    fn choose(&self) -> Option<&Self::Item> {
        if self.is_empty() {
            None
        } else {
            let idx = rand::<usize>() % self.len();
            Some(&self[idx])
        }
    }

    fn get_random_sample(&self) -> WallSwitchResult<Self::Item>
    where
        Self::Item: Copy,
    {
        self.choose().copied().ok_or(WallSwitchError::EmptySlice)
    }

    fn get_random_sample_cloned(&self) -> WallSwitchResult<Self::Item>
    where
        Self::Item: Clone,
    {
        self.choose().cloned().ok_or(WallSwitchError::EmptySlice)
    }
}

// --- Rand (Unificado) --- //

/// Helper function to generate a raw 64-bit unsigned integer using the system's `RandomState`.
#[inline]
fn raw_u64() -> u64 {
    RandomState::new().build_hasher().finish()
}

/// A trait for types that can be generated randomly.
pub trait Rand {
    /// Generates a random instance of the implementing type.
    fn rand() -> Self;
}

macro_rules! impl_rand_int {
    ($($t:ty),*) => {
        $(
            impl Rand for $t {
                #[inline]
                fn rand() -> Self {
                    raw_u64() as $t
                }
            }
        )*
    };
}

// Implements Rand for all standard integer types by casting the raw u64 value.
impl_rand_int!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

impl Rand for f64 {
    #[inline]
    fn rand() -> Self {
        (raw_u64() as f64) / (u64::MAX as f64)
    }
}

impl Rand for f32 {
    #[inline]
    fn rand() -> Self {
        (raw_u64() as f32) / (u64::MAX as f32)
    }
}

/// Generates a random value of type `T`.
///
/// For integer types, this generates a value across the entire range of the type.
/// For floating-point types (`f32` and `f64`), this generates a value in the range `[0.0, 1.0)`.
///
/// # Examples
///
/// ```
/// use wallswitch::random::rand;
///
/// let random_u32: u32 = rand();
/// let random_float: f64 = rand();
/// assert!(random_float >= 0.0 && random_float < 1.0);
/// ```
#[inline]
pub fn rand<T: Rand>() -> T {
    T::rand()
}

// --- Random Cast --- //

/// Helper trait to handle type-safe, generic casting for random integer outputs.
pub trait RandomCast {
    /// Converts a raw 64-bit unsigned integer into the target numeric type.
    fn from_u64(val: u64) -> Self;
}

macro_rules! impl_random_cast {
    ($($t:ty),*) => {
        $(
            impl RandomCast for $t {
                #[inline]
                fn from_u64(val: u64) -> Self {
                    val as $t
                }
            }
        )*
    };
}

impl_random_cast!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

// --- Get Random Integer --- //

/// Generate a random integer value in the given range (min, max) inclusive.
///
/// Accepts signed, unsigned, and platform-specific integer types for boundaries,
/// and automatically casts the result to the inferred numeric return type.
///
/// # Examples
///
/// ```
/// use wallswitch::get_random_integer;
///
/// let angle: f64 = get_random_integer(0, 359);
/// let idx: usize = get_random_integer(0, 10);
/// let iter: u32 = get_random_integer(300, 990);
/// ```
pub fn get_random_integer<T, R>(min: T, max: T) -> R
where
    T: TryInto<u64>,
    R: RandomCast,
{
    let min_val = min.try_into().ok().unwrap_or(0);
    let max_val = max.try_into().ok().unwrap_or(0);

    let raw_val = if min_val >= max_val {
        min_val
    } else {
        min_val + rand::<u64>() % (max_val - min_val + 1)
    };

    R::from_u64(raw_val)
}

/// Generate a random integer value in the given range (min, max) inclusive.
///
/// Returns an error if `min > max` and automatically casts the output to the inferred type.
pub fn get_random_integer_safe<R>(min: u64, max: u64) -> WallSwitchResult<R>
where
    R: RandomCast,
{
    if min > max {
        Err(WallSwitchError::MinMax { min, max })
    } else {
        let raw_val = min + rand::<u64>() % (max - min + 1);
        Ok(R::from_u64(raw_val))
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_random {
    use super::*;

    #[test]
    fn test_shuffle_and_rand() {
        let mut data: Vec<usize> = (0..100).collect();
        let original = data.clone();
        data.shuffle();

        assert_eq!(data.len(), 100);
        // It is statistically highly improbable for a shuffled 100-element vector to remain unchanged
        assert_ne!(data, original);
    }

    #[test]
    fn test_choose() {
        let data: Vec<usize> = (0..10).collect();
        let chosen = data.choose();
        assert!(chosen.is_some());
        assert!(data.contains(chosen.unwrap()));

        let empty_data: Vec<usize> = vec![];
        assert!(empty_data.choose().is_none());
    }

    #[test]
    fn test_automatic_inference_casts() {
        // Automatically infers and converts to diverse destination types
        let val_usize: usize = get_random_integer(10, 20);
        assert!((10..=20).contains(&val_usize));

        let val_u32: u32 = get_random_integer(100, 200);
        assert!((100..=200).contains(&val_u32));

        let val_f64: f64 = get_random_integer(0, 359);
        assert!((0.0..=359.0).contains(&val_f64));

        let val_i32: i32 = get_random_integer(1, 5);
        assert!((1..=5).contains(&val_i32));
    }

    #[test]
    fn test_safe_random_bounds() {
        let valid: Result<u32, _> = get_random_integer_safe(50, 100);
        assert!(valid.is_ok());
        let val = valid.unwrap();
        assert!((50..=100).contains(&val));

        let invalid: Result<u32, _> = get_random_integer_safe(100, 50);
        assert!(invalid.is_err());
    }

    #[test]
    fn test_get_random_sample_success() {
        let data = [42, 100, 200];
        let result: WallSwitchResult<i32> = data.get_random_sample();

        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(data.contains(&value));
    }

    #[test]
    fn test_get_random_sample_empty() {
        let empty_data: Vec<i32> = vec![];
        let result: WallSwitchResult<i32> = empty_data.get_random_sample();

        assert!(result.is_err());
        match result {
            Err(WallSwitchError::EmptySlice) => {} // Sucesso: o erro correto foi retornado
            _ => panic!("Expected WallSwitchError::EmptySlice, but got a different result"),
        }
    }
}
