use crate::{ConcurrencyExt, Dimension, DimensionError, FileInfo, WallSwitchResult};
use blake3::Hasher;
use image::ImageReader;
use std::{
    fs::File,
    io::{BufReader, Error, Read},
    path::PathBuf,
    thread,
};

/// Size of the buffer used for reading files during the hashing process.
/// 64 KB is an optimal balance between memory usage and disk read throughput.
const BUFFER_SIZE: usize = 64 * 1024;

/// Probes image dimensions using pure-Rust in-process header scanning.
///
/// This function uses content-based format detection (magic bytes) rather than
/// relying strictly on the file extension, making it robust against missing or
/// incorrect file extensions.
pub fn probe_image_dimension(path: &PathBuf) -> WallSwitchResult<Dimension> {
    // If opening or reading the file fails, propagate automatically as WallSwitchError::Io
    let reader = ImageReader::open(path)?.with_guessed_format()?;

    // If decoding the dimensions fails, map to DimensionError::ReadFailed with path and source context
    let (width, height) = reader
        .into_dimensions()
        .map_err(|err| DimensionError::ReadFailed {
            path: path.clone(),
            source: err,
        })?;

    Ok(Dimension {
        width: width as u64,
        height: height as u64,
    })
}

// Helper function to process a single file.
// Standard io::Error is automatically converted to WallSwitchError::Io via '?'
fn compute_single_hash(file_info: &mut FileInfo) -> WallSwitchResult<()> {
    // Standard library's File::open returns std::io::Error.
    // The '?' operator implicitly converts it to WallSwitchError::Io.
    let file = File::open(&file_info.path)?;

    let reader = BufReader::with_capacity(BUFFER_SIZE, file);

    // Map custom hashing errors to a standard io::Error, which is then promoted via '?'
    let hash = get_hash(reader).map_err(|err| Error::other(err.to_string()))?;

    file_info.hash = hash;
    Ok(())
}

/// Computes the BLAKE3 hash of multiple files using a thread-safe parallel approach.
pub fn compute_hashes_parallel(files: &mut [FileInfo]) {
    let chunk_size = files.get_chunk_size(files.len());

    thread::scope(|scope| {
        for chunk in files.chunks_mut(chunk_size) {
            scope.spawn(move || {
                for file_info in chunk {
                    if let Err(err) = compute_single_hash(file_info) {
                        // Keeps the error log clear, accurate, and localized
                        eprintln!(
                            "Failed to compute BLAKE3 hash for '{}': {}",
                            file_info.path.display(),
                            err
                        );
                    }
                }
            });
        }
    });
}

/// Calculates the BLAKE3 hash from any IO Reader stream.
pub fn get_hash(mut reader: impl Read) -> WallSwitchResult<String> {
    let mut hasher = Hasher::new();
    let mut buffer = [0_u8; BUFFER_SIZE];

    loop {
        let count = reader.read(&mut buffer)?;

        if count == 0 {
            break;
        }

        hasher.update(&buffer[..count]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_metadata {
    use super::*;
    use image::{ImageFormat, RgbImage};
    use std::{env, fs};

    #[test]
    fn test_probe_image_dimension_supported_formats() {
        // Formats natively supported and encoded by the `image` crate by default
        let formats_to_test = vec![
            (ImageFormat::Png, "png"),
            (ImageFormat::Jpeg, "jpg"),
            (ImageFormat::WebP, "webp"),
            (ImageFormat::Tiff, "tif"),
        ];

        let temp_dir = env::temp_dir();
        let expected_width = 120;
        let expected_height = 80;

        for (format, ext) in formats_to_test {
            let file_path = temp_dir.join(format!("wallswitch_test_probe.{ext}"));

            // 1. Generate a valid minimal image in-memory
            let img = RgbImage::new(expected_width, expected_height);

            // 2. Save with the target format, skipping if the encoder feature is missing
            if let Err(err) = img.save_with_format(&file_path, format) {
                eprintln!(
                    "Skipping format {:?}: encoder failed or missing feature. Details: {}",
                    format, err
                );
                continue;
            }

            // 3. Test probe_image_dimension
            match probe_image_dimension(&file_path) {
                Ok(dim) => assert_eq!(
                    dim,
                    Dimension {
                        width: expected_width as u64,
                        height: expected_height as u64,
                    }
                ),
                Err(err) => panic!("Expected Ok for format {format:?}, but got error: {err:?}"),
            }

            // 4. Cleanup the temporary file
            let _ = fs::remove_file(file_path);
        }
    }

    #[test]
    fn test_probe_image_dimension_invalid_file() {
        let temp_dir = env::temp_dir();
        let file_path = temp_dir.join("wallswitch_invalid_test.png");

        // Write magic bytes only (incomplete file structure) to test failure path
        let magic_bytes_only = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        fs::write(&file_path, magic_bytes_only).unwrap();

        let result = probe_image_dimension(&file_path);
        assert!(
            result.is_err(),
            "Expected dimension probing to fail on incomplete header file"
        );

        let _ = fs::remove_file(file_path);
    }
}
