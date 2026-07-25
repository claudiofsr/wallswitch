//! Recursive directory traversal using the `walkdir` crate.
//!
//! This module efficiently scans directories for files matching specific extensions
//! and extracts metadata (size, modification time) to build a list of [`FileInfo`] objects.
//! It leverages `walkdir` for robust handling of file system edge cases compared
//! to manual `std::fs` recursion.

use crate::{Config, FileInfo, WallSwitchResult};
use std::path::Path;
use walkdir::WalkDir;

/// Retrieves and filters file information from a specified directory.
///
/// This function traverses the directory recursively, filtering files by the extensions
/// specified in the provided [`Config`]. It extracts metadata such as file size and
/// modification time, returning a list of [`FileInfo`] structs.
///
/// # Arguments
///
/// * `path` - A path reference to the directory to be searched.
/// * `config` - A reference to the configuration containing the allowed file extensions.
///
/// # Errors
///
/// Returns a [`WallSwitchResult`] if the directory traversal or metadata retrieval fails
/// in a way that prevents the operation from completing. Internal I/O errors on individual
/// files or subdirectories are safely skipped.
///
/// # Examples
///
/// ```ignore
/// use wallswitch::{get_files_from_directory, Config};
/// use std::path::Path;
///
/// let config = Config { extensions: vec!["jpg".to_string(), "png".to_string()] };
/// let files = get_files_from_directory(Path::new("./images"), &config)?;
/// ```
pub fn get_files_from_directory<P>(path: P, config: &Config) -> WallSwitchResult<Vec<FileInfo>>
where
    P: AsRef<Path>,
{
    let extensions = &config.extensions;

    let mut all_files: Vec<FileInfo> = WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        // Safely ignore directory traversal errors
        .filter_map(Result::ok)
        // Ensure the entry is a file
        .filter(|entry| entry.file_type().is_file())
        // Filter by the allowed extensions (case-insensitive and zero heap allocation)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext_str| {
                    extensions
                        .iter()
                        .any(|allowed_ext| ext_str.eq_ignore_ascii_case(allowed_ext))
                })
        })
        // Map the DirEntry to FileInfo, ignoring entries with inaccessible metadata
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let size = metadata.len();

            // Retrieve the modification time as a Unix timestamp, defaulting to 0 if invalid
            let mtime = metadata
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            Some(FileInfo {
                size,
                mtime,
                path: entry.into_path(),
                ..Default::default()
            })
        })
        .collect();

    // In-place unstable sort. Since path strings are unique,
    // an unstable sort is faster and does not allocate auxiliary memory.
    all_files.sort_unstable_by(|a, b| a.path.cmp(&b.path));

    Ok(all_files)
}
