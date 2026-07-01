//! # System Environment Lifecycle Design
//!
//! To prevent performance bottlenecks caused by redundant system calls and environment queries,
//! `wallswitch` implements a single-instantiation pattern for environment metadata.
//!
//! ```text
//!              +--------------------+
//!              |  app::run() entry  |
//!              +---------+----------+
//!                        |
//!                        | Instantiates single Environment instance
//!                        v
//!              +---------+----------+
//!              |  Environment::new  |
//!              +---------+----------+
//!                        |
//!         +--------------+---------------+
//!         | Shared reference (&env)      |
//!         v                              v
//! +-------+--------+             +-------+--------+
//! | Arguments::build|             |  Config::new   |
//! +----------------+             +-------+--------+
//!                                        |
//!                                        v
//!                                +-------+--------+
//!                                |   State::load  |
//!                                +----------------+
//! ```
//!
//! This architecture guarantees that platform-specific I/O queries (such as retrieving
//! user home directories, temp folder locations, or AppData parameters) are executed
//! exactly once during bootstrap, sharing the immutable context with down-stack modules.

use crate::WallSwitchResult;
use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
};

#[cfg(unix)]
const DEFAULT_TEMP_DIR: &str = "/tmp";
#[cfg(windows)]
const DEFAULT_TEMP_DIR: &str = "C:\\Windows\\Temp";
#[cfg(not(any(unix, windows)))]
const DEFAULT_TEMP_DIR: &str = ".";

#[cfg(unix)]
const DEFAULT_HOME_DIR: &str = "/";
#[cfg(windows)]
const DEFAULT_HOME_DIR: &str = "C:\\Users";
#[cfg(not(any(unix, windows)))]
const DEFAULT_HOME_DIR: &str = ".";

/// Environment variables and system metadata.
///
/// Uses Cow (Copy-on-Write) to handle both owned paths/strings from the environment
/// and static fallbacks efficiently without unnecessary allocations.
pub struct Environment<'a> {
    pub home_dir: Cow<'a, Path>,
    pub temp_dir: Cow<'a, Path>,
    pub cache_dir: Cow<'a, Path>,
    pub pkg_name: Cow<'a, str>,
}

impl Environment<'_> {
    /// Returns a fallback environment config with static defaults.
    pub fn fallback() -> Environment<'static> {
        let home = Path::new(DEFAULT_HOME_DIR);
        Environment {
            home_dir: Cow::Borrowed(Path::new(home)),
            temp_dir: Cow::Borrowed(Path::new(DEFAULT_TEMP_DIR)),
            cache_dir: Cow::Owned(fetch_cache_dir(home)),
            pkg_name: Cow::Borrowed("wallswitch"),
        }
    }

    /// Initializes the environment by gathering data from system variables.
    ///
    /// This resolves the system home directory, temporary directory, and package name safely.
    pub fn new() -> WallSwitchResult<Environment<'static>> {
        let home_dir = fetch_home_dir();
        let temp_dir = fetch_temp_dir();
        let cache_dir = fetch_cache_dir(&home_dir);
        let pkg_name = fetch_pkg_name();

        Ok(Environment {
            home_dir: Cow::Owned(home_dir),
            temp_dir: Cow::Owned(temp_dir),
            cache_dir: Cow::Owned(cache_dir),
            pkg_name: Cow::Owned(pkg_name),
        })
    }

    /// Returns the standard configuration directory based on target OS platform guidelines.
    ///
    /// - On Unix-like systems (Linux/macOS), returns `~/.config/wallswitch`.
    /// - On Windows systems, returns `%APPDATA%\wallswitch`.
    pub fn get_app_config_dir(&self) -> PathBuf {
        #[cfg(windows)]
        {
            // Windows standard: AppData\Roaming
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| self.home_dir.join("AppData").join("Roaming"))
                .join(&self.pkg_name)
        }
        #[cfg(not(windows))]
        {
            // Unix standard: ~/.config
            self.home_dir.join(".config").join(&*self.pkg_name)
        }
    }

    /// Returns the standard non-volatile user cache directory based on OS guidelines.
    ///
    /// - On Unix-like systems, returns `~/.cache/wallswitch`.
    /// - On Windows systems, returns `%LOCALAPPDATA%\wallswitch`.
    pub fn get_app_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(self.pkg_name.as_ref())
    }

    /// Returns a reference to the home directory path.
    pub fn get_home_dir(&self) -> &Path {
        &self.home_dir
    }

    /// Returns a reference to the temporary directory path.
    pub fn get_temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Returns a reference to the package name.
    pub fn get_pkg_name(&self) -> &str {
        &self.pkg_name
    }
}

/// Fetches the system's temporary directory using platform-specific APIs.
fn fetch_temp_dir() -> PathBuf {
    env::temp_dir()
}

/// Safely fetches the package name, defaulting to "wallswitch".
fn fetch_pkg_name() -> String {
    env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "wallswitch".to_string())
}

/// Resolves the user's home directory across different operating systems,
/// falling back to the temporary directory if none is found.
fn fetch_home_dir() -> PathBuf {
    #[cfg(unix)]
    {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
    }
    #[cfg(windows)]
    {
        env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir)
    }
    #[cfg(not(any(unix, windows)))]
    {
        env::temp_dir()
    }
}

/// Resolves the standard user cache folder.
fn fetch_cache_dir(home_dir: &Path) -> PathBuf {
    #[cfg(unix)]
    {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(".cache"))
    }
    #[cfg(target_os = "windows")]
    {
        env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join("AppData").join("Local"))
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        env::temp_dir()
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_initialization() {
        let env = Environment::fallback();
        assert!(!env.get_home_dir().as_os_str().is_empty());
        assert!(!env.get_temp_dir().as_os_str().is_empty());
        assert_eq!(env.get_pkg_name(), "wallswitch");
    }

    #[test]
    fn test_new_initialization() {
        let env_result = Environment::new();
        assert!(env_result.is_ok());

        let env = env_result.unwrap();
        assert!(env.get_home_dir().exists());
        assert!(
            env.get_temp_dir().exists() || cfg!(any(target_os = "none", target_arch = "wasm32"))
        );
    }
}
