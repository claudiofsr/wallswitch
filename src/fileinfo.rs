use crate::{Config, Countable, Dimension, DimensionError, MyResult, WSError, exec_cmd};
use std::{
    fmt,
    fs::File,
    hash::{DefaultHasher, Hasher},
    io::{BufReader, Read},
    path::PathBuf,
    process::Command,
    thread,
};

const BUFFER_SIZE: usize = 64 * 1024;

/// Image information
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct FileInfo {
    /// File number (index + 1)
    pub number: usize,
    /// Total File number
    pub total: usize,
    /// dimension: width x length of an image
    pub dimension: Dimension,
    /// The size of the file, in bytes
    pub size: u64,
    /// AHash from Path
    pub hash: String,
    /// The image file path
    pub path: PathBuf,
}

impl FileInfo {
    /**
    Returns true if the given pattern matches a sub-slice of this path.

    Returns false if it does not.
    */
    pub fn path_contains(&self, string: &str) -> bool {
        match self.path.to_str() {
            Some(p) => p.contains(string),
            None => false,
        }
    }

    /// Update dimension field and valid_dimension field
    pub fn update_info(&mut self, config: &Config) -> MyResult<()> {
        // identify -format %wx%h image_file_path
        let mut cmd = Command::new("identify");
        let identify_cmd = cmd
            .arg("-format")
            .arg("%wx%h") // x separator
            .arg(&self.path);

        let identify_out = exec_cmd(identify_cmd, config.verbose, "identify")?;

        let sdt_output = String::from_utf8(identify_out.stdout)?;

        self.dimension = Dimension::new(&sdt_output)?;

        Ok(())
    }

    /// Check if the dimension is valid.
    pub fn dimension_is_valid(&self, config: &Config) -> bool {
        let is_valid = self.dimension.is_valid(config);

        if !is_valid {
            let dim_error = DimensionError {
                dimension: self.dimension.clone(),
                log_min: self.dimension.get_log_min(config),
                log_max: self.dimension.get_log_max(config),
                path: self.path.clone(),
            };

            //return Err(WSError::InvalidDim(dim_error));
            eprintln!("{}", WSError::InvalidDimension(dim_error));
        }

        is_valid
    }

    /// Check if the size is valid.
    pub fn size_is_valid(&self, config: &Config) -> bool {
        self.size >= config.min_size && self.size <= config.max_size
    }

    pub fn name_is_valid(&self, config: &Config) -> bool {
        let is_valid = self.path.file_name() != config.wallpaper.file_name();

        if !is_valid && let Some(path) = self.path.file_name() {
            eprintln!("{}\n", WSError::InvalidFilename(path.into()));
        }

        is_valid
    }
}

/// FileInfo Extension
pub trait FileInfoExt {
    fn get_width_min(&self) -> Option<u64>;
    fn get_max_size(&self) -> Option<u64>;
    fn get_max_number(&self) -> Option<usize>;
    fn get_max_dimension(&self) -> Option<u64>;
    fn sizes_are_valid(&self, config: &Config) -> bool;
    fn update_number(&mut self);
    fn update_hash(&mut self) -> MyResult<()>;
}

impl FileInfoExt for [FileInfo] {
    fn get_width_min(&self) -> Option<u64> {
        self.iter().map(|file_info| file_info.dimension.width).min()
    }

    fn get_max_size(&self) -> Option<u64> {
        self.iter().map(|file_info| file_info.size).max()
    }

    fn get_max_number(&self) -> Option<usize> {
        self.iter().map(|file_info| file_info.number).max()
    }

    fn get_max_dimension(&self) -> Option<u64> {
        self.iter()
            .map(|file_info| file_info.dimension.maximum())
            .max()
    }

    fn sizes_are_valid(&self, config: &Config) -> bool {
        self.iter().all(|file_info| {
            let is_valid = file_info.size_is_valid(config);

            if !is_valid {
                let size = file_info.size;

                let min_size = config.min_size;
                let max_size = config.max_size;

                let path = file_info.path.clone();

                // Print Indented file information
                print!("{}", SliceDisplay(self));

                eprintln!("{}", WSError::InvalidSize(min_size, size, max_size));
                eprintln!("{}\n", WSError::DisregardPath(path));
            }

            is_valid
        })
    }

    /// Update FileInfo number field
    fn update_number(&mut self) {
        let total = self.len();
        self.iter_mut().enumerate().for_each(|(index, file)| {
            file.number = index + 1;
            file.total = total;
        });
    }

    /// Update FileInfo hash field
    fn update_hash(&mut self) -> MyResult<()> {
        // Parallelize the computation using std::thread::scope
        thread::scope(|scope| {
            for file_info in self {
                scope.spawn(move || -> MyResult<()> {
                    // let id = thread::current().id();
                    // println!("identifier thread: {id:?}");

                    let file = File::open(&file_info.path)?;
                    let reader = BufReader::with_capacity(BUFFER_SIZE, file);
                    let hash = get_hash(reader)?;

                    // println!("path: '{}' ; hash: '{}'", file_info.path.display(), hash);

                    file_info.hash = hash;

                    Ok(())
                });
            }
        });

        Ok(())
    }
}

/// Calculates the hash from Path.
///
/// If the same stream of bytes is fed into each hasher, the same output will also be generated.
///
/// <https://doc.rust-lang.org/std/hash/trait.Hasher.html>
pub fn get_hash(mut reader: impl Read) -> MyResult<String> {
    let mut buffer = [0_u8; BUFFER_SIZE];
    let mut hasher = DefaultHasher::new();

    loop {
        // read up to BUFFER_SIZE bytes to buffer
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish().to_string())
}

/// Implement fmt::Display for Slice `[T]`
///
/// <https://stackoverflow.com/questions/30633177/implement-fmtdisplay-for-vect>
///
/// <https://stackoverflow.com/questions/33759072/why-doesnt-vect-implement-the-display-trait>
///
/// <https://gist.github.com/hyone/d6018ee1ac8f9496fed839f481eb59d6>
pub struct SliceDisplay<'a>(pub &'a [FileInfo]);

impl fmt::Display for SliceDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The number of digits of maximum values
        let digits_n: Option<usize> = self.0.get_max_number().map(|n| n.count_chars());
        let digits_s: Option<usize> = self.0.get_max_size().map(|s| s.count_chars());
        let digits_d: Option<usize> = self.0.get_max_dimension().map(|d| d.count_chars());

        match (digits_n, digits_s, digits_d) {
            (Some(num_digits_number), Some(num_digits_size), Some(num_digits_dimension)) => {
                for file in self.0 {
                    let dim = format!(
                        "Dimension {{ width: {width:>d$}, height: {height:>d$} }}",
                        width = file.dimension.width,
                        height = file.dimension.height,
                        d = num_digits_dimension,
                    );

                    writeln!(
                        f,
                        "images[{number:0n$}/{t}]: {dim}, size: {size:>s$}, path: {p:?}",
                        number = file.number,
                        n = num_digits_number,
                        t = file.total,
                        size = file.size,
                        s = num_digits_size,
                        p = file.path,
                    )?;
                }
            }
            _ => return Err(std::fmt::Error),
        }

        Ok(())
    }
}

#[cfg(test)]
mod test_info {
    #[test]
    /// `cargo test -- --show-output get_min_value_of_vec`
    fn get_min_value_of_vec_v1() {
        let values: Vec<i32> = vec![5, 6, 8, 4, 2, 7];

        let min_value: Option<i32> = values.iter().min().copied();

        println!("values: {values:?}");
        println!("min_value: {min_value:?}");

        assert_eq!(min_value, Some(2));
    }

    #[test]
    /// `cargo test -- --show-output get_min_value_of_vec`
    ///
    /// <https://stackoverflow.com/questions/58669865/how-to-get-the-minimum-value-within-a-vector-in-rust>
    fn get_min_value_of_vec_v2() {
        let values: Vec<i32> = vec![5, 6, 8, 4, 2, 7];

        // The empty vector must be filtered beforehand!
        // let values: Vec<i32> = vec![]; // Not work!!!

        // Get the minimum value without being wrapped by Option<T>
        let min_value: i32 = values
            .iter()
            //.into_iter()
            //.fold(i32::MAX, i32::min);
            .fold(i32::MAX, |arg0: i32, other: &i32| i32::min(arg0, *other));

        println!("values: {values:?}");
        println!("min_value: {min_value}");

        assert_eq!(min_value, 2);
    }
}
