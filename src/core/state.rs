use crate::{Dimension, WallSwitchResult, get_config_path};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

const MAX_ITENS: usize = 10_000;

/// Represents cached information of a file to prevent re-hashing and re-probing
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CacheEntry {
    pub size: u64,
    pub mtime: u64,
    pub hash: String,
    #[serde(default)]
    pub dimension: Option<Dimension>,
}

/// Manages the Wallpaper History and the Smart Cache
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub history: Vec<PathBuf>,
    pub hashes: HashMap<PathBuf, CacheEntry>,
}

impl State {
    /// Loads the state from the JSON file. If it doesn't exist or is invalid, returns the default.
    pub fn load() -> Self {
        if let Ok(path) = Self::get_path()
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(state) = serde_json::from_str(&content)
        {
            return state;
        }
        State::default()
    }

    /// Persists the current image history and hash cache to disk.
    ///
    /// Prevents the history from exceeding MAX_ITEMS to optimize performance.
    pub fn save(&mut self) -> WallSwitchResult<()> {
        // Limits the history to the last MAX_ITENS to prevent infinite growth
        if self.history.len() > MAX_ITENS {
            let start = self.history.len() - MAX_ITENS;
            self.history = self.history[start..].to_vec();
        }

        let path = Self::get_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::File::create(path)?;
        serde_json::to_writer_pretty(file, self)?;
        Ok(())
    }

    /// Clears cache paths that no longer exist (Garbage Collection)
    pub fn garbage_collect(&mut self) {
        self.hashes.retain(|path, _| path.exists());
    }

    /// Returns the state file path (~/.config/wallswitch/wallswitch-state.json)
    fn get_path() -> WallSwitchResult<PathBuf> {
        let mut path = get_config_path()?;
        path.set_file_name("wallswitch-state.json");
        Ok(path)
    }
}
