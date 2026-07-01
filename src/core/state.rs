use crate::{
    AtomicWriteExt as _, Dimension, Environment, WallSwitchError, WallSwitchResult, get_config_path,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

const MAX_ITENS: usize = 10_000;

/// Represents cached metadata of an image file to prevent redundant hashing and dimension probing.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CacheEntry {
    pub size: u64,
    pub mtime: u64,
    pub hash: String,
    #[serde(default)]
    pub dimension: Option<Dimension>,
}

/// Manages the persistence of the wallpaper history loop and the smart file cache.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct State {
    pub history: Vec<PathBuf>,
    pub hashes: HashMap<PathBuf, CacheEntry>,
}

impl State {
    /// Loads the persistent state file from the system configuration path.
    pub fn load(env: &Environment) -> Self {
        if let Ok(path) = Self::get_path(env)
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(state) = serde_json::from_str(&content)
        {
            return state;
        }
        State::default()
    }

    /// Atomically persists the history loops and image metadata cache to disk.
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchResult`] if writing the file fails.
    pub fn save(&mut self, env: &Environment) -> WallSwitchResult<()> {
        if self.history.len() > MAX_ITENS {
            let start = self.history.len() - MAX_ITENS;
            self.history = self.history[start..].to_vec();
        }

        let path = Self::get_path(env)?;

        path.atomic_write(|temp_path| {
            let file =
                fs::File::create(temp_path).map_err(|io_error| WallSwitchError::IOError {
                    path: temp_path.to_path_buf(),
                    io_error,
                })?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, self)?;
            writer.flush()?;
            Ok(())
        })?;

        Ok(())
    }

    /// Removes untracked paths that no longer exist on the current filesystem from the cache.
    pub fn garbage_collect(&mut self) {
        self.hashes.retain(|path, _| path.exists());
        self.hashes.shrink_to_fit();
    }

    /// Resolves the absolute path to the application state JSON file.
    fn get_path(env: &Environment) -> WallSwitchResult<PathBuf> {
        let mut path = get_config_path(env)?;
        path.set_file_name("wallswitch-state.json");
        Ok(path)
    }
}
