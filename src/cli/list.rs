use crate::{CacheEntry, Colors, FileInfo, SliceDisplay, State, WallSwitchResult};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortCriteria {
    Path,
    Size,
    SizeDesc,
    Name,
    Extension,
    Width,
    Height,
    Area,
    Ratio,
    Time,
    Processed,   // Dimension is not null
    Unprocessed, // Dimension is null
    Cache,       // Everything in the cache
}

impl FromStr for SortCriteria {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "path" => Ok(Self::Path),
            "size" => Ok(Self::Size),
            "sizedesc" => Ok(Self::SizeDesc),
            "name" => Ok(Self::Name),
            "extension" => Ok(Self::Extension),
            "width" => Ok(Self::Width),
            "height" => Ok(Self::Height),
            "area" => Ok(Self::Area),
            "ratio" => Ok(Self::Ratio),
            "time" => Ok(Self::Time),
            "processed" => Ok(Self::Processed),
            "unprocessed" => Ok(Self::Unprocessed),
            "cache" => Ok(Self::Cache),
            _ => Err("Invalid criteria. \
            Use: path, size, sizedesc, name, \
            extension, width, height, area, ratio, \
            time, processed, unprocessed, cache"
                .to_string()),
        }
    }
}

/// Lists the internal state cache as a filtered JSON object.
///
/// This is useful for debugging the Smart Cache or for integration with
/// external tools like 'jq'.
pub fn list_json_cache(state: &State, criteria: SortCriteria) -> WallSwitchResult<()> {
    // Collect filtered references to avoid cloning heavy data before serialization
    let filtered: HashMap<&PathBuf, &CacheEntry> = state
        .hashes
        .iter()
        .filter(|(_, entry)| match criteria {
            SortCriteria::Processed => entry.dimension.is_some(),
            SortCriteria::Unprocessed => entry.dimension.is_none(),
            SortCriteria::Cache => true,
            _ => false, // Standard sort criteria are handled by list_all_images
        })
        .collect();

    // Serialize the resulting map to a pretty-printed JSON string
    let json = serde_json::to_string_pretty(&filtered)?;
    println!("{json}");

    Ok(())
}

/// Displays images in a human-readable table format with sorting.
pub fn list_all_images(mut images: Vec<FileInfo>, criteria: SortCriteria) -> WallSwitchResult<()> {
    let label = match criteria {
        SortCriteria::Path => {
            images.sort_by(|a, b| a.path.cmp(&b.path));
            "PATH"
        }
        SortCriteria::Size => {
            images.sort_by_key(|f| f.size);
            "SIZE (ASC)"
        }
        SortCriteria::SizeDesc => {
            images.sort_by_key(|b| std::cmp::Reverse(b.size));
            "SIZE (DESC)"
        }
        SortCriteria::Name => {
            images.sort_by(|a, b| a.path.file_name().cmp(&b.path.file_name()));
            "FILE NAME"
        }
        SortCriteria::Extension => {
            images.sort_by(|a, b| a.path.extension().cmp(&b.path.extension()));
            "EXTENSION"
        }
        SortCriteria::Width => {
            images.sort_by_key(|f| f.dimension.as_ref().map(|d| d.width).unwrap_or(0));
            "WIDTH"
        }
        SortCriteria::Height => {
            images.sort_by_key(|f| f.dimension.as_ref().map(|d| d.height).unwrap_or(0));
            "HEIGHT"
        }
        SortCriteria::Area => {
            images.sort_by_key(|f| {
                f.dimension
                    .as_ref()
                    .map(|d| d.width * d.height)
                    .unwrap_or(0)
            });
            "AREA (TOTAL PIXELS)"
        }
        SortCriteria::Ratio => {
            images.sort_by(|a, b| {
                let r_a = a
                    .dimension
                    .as_ref()
                    .map(|d| d.width as f64 / d.height as f64)
                    .unwrap_or(0.0);
                let r_b = b
                    .dimension
                    .as_ref()
                    .map(|d| d.width as f64 / d.height as f64)
                    .unwrap_or(0.0);
                r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
            });
            "ASPECT RATIO"
        }
        SortCriteria::Time => {
            images.sort_by_key(|f| f.mtime);
            "MODIFICATION TIME (NEWEST LAST)"
        }
        // Fallback for criteria handled by list_json_cache
        _ => "CACHE STATE",
    };

    println!("Listing images sorted by {}:", label.yellow().bold());

    let total = images.len();
    for (i, img) in images.iter_mut().enumerate() {
        img.number = i + 1;
        img.total = total;
    }

    print!("{}", SliceDisplay(&images));
    println!("\nTotal images found: {}", total.to_string().green().bold());
    Ok(())
}
