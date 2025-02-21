use crate::{Config, FileInfo, MyResult};
//use rayon::prelude::*;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

/// Get all files into one vector.
///
/// Use walkdir.
pub fn get_files_from_directory<P>(path: P, config: &Config) -> MyResult<Vec<FileInfo>>
where
    P: AsRef<Path>,
{
    let entries: Vec<DirEntry> = get_image_entries(path, config)?;

    let all_files: Vec<FileInfo> = entries
        //.into_par_iter() // rayon parallel iterator
        .into_iter()
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            let size = metadata.len();
            //let extension = metadata.file_type();

            Some(FileInfo {
                size,
                path: entry.into_path(),
                ..Default::default()
            })
        })
        .collect();

    Ok(all_files)
}

/// Get result: Vec<DirEntry>.
fn get_image_entries<P>(path: P, config: &Config) -> MyResult<Vec<DirEntry>>
where
    P: AsRef<Path>,
{
    // default extensions: ["avif", "jpg", "jpeg", "png", "svg", "tif", "webp"]
    let extensions = &config.extensions;

    let entries: Vec<DirEntry> = WalkDir::new(path)
        .sort_by_file_name()
        .into_iter()
        .flatten() // Result<DirEntry, Error> to DirEntry
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| {
                extensions
                    .iter()
                    .any(|extension| ext.eq_ignore_ascii_case(extension))
            })
        })
        .collect();

    Ok(entries)
}
