use crate::{ConcurrencyExt, Dimension, FileInfo, WallSwitchError, WallSwitchResult};
use blake3::Hasher;
use image::image_dimensions;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    thread,
};

/// Size of the buffer used for reading files during the hashing process.
/// 64 KB is an optimal balance between memory usage and disk read throughput.
const BUFFER_SIZE: usize = 64 * 1024;

/// Probes image dimensions using pure-Rust in-process header scanning.
pub fn probe_image_dimension(path: &PathBuf, _verbose: bool) -> WallSwitchResult<Dimension> {
    match image_dimensions(path) {
        Ok((width, height)) => Ok(Dimension {
            width: width as u64,
            height: height as u64,
        }),
        Err(err) => Err(WallSwitchError::UnableToFind(format!(
            "Failed to probe image dimensions for {}: {}",
            path.display(),
            err
        ))),
    }
}

/// Computes the BLAKE3 hash of multiple files using a thread-safe parallel approach.
pub fn compute_hashes_parallel(files: &mut [FileInfo]) {
    let chunk_size = files.get_chunk_size(files.len());

    thread::scope(|scope| {
        for chunk in files.chunks_mut(chunk_size) {
            scope.spawn(move || {
                for file_info in chunk {
                    if let Ok(file) = File::open(&file_info.path) {
                        let reader = BufReader::with_capacity(BUFFER_SIZE, file);

                        if let Ok(hash) = get_hash(reader) {
                            file_info.hash = hash;
                        }
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
