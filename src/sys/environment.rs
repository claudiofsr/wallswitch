use crate::{WallSwitchError, WallSwitchResult};
use std::{borrow::Cow, env};

/// Environment variables and system metadata.
///
/// Uses Cow (Copy-on-Write) to handle both owned strings from the environment
/// and static strings from fallbacks efficiently.
pub struct Environment<'a> {
    pub desktop: Cow<'a, str>,
    pub home: Cow<'a, str>,
    pub pkg_name: Cow<'a, str>,
}

impl Environment<'_> {
    /// Provides a safe fallback environment if system variables are missing.
    /// This prevents the application from crashing in restricted environments.
    pub fn fallback() -> Environment<'static> {
        Environment {
            desktop: Cow::Borrowed("openbox"),
            home: Cow::Borrowed("/tmp"),
            pkg_name: Cow::Borrowed("wallswitch"),
        }
    }

    /// Initializes the environment by gathering data from system variables.
    /// Returns a WallSwitchResult instead of panicking if critical data is missing.
    pub fn new() -> WallSwitchResult<Environment<'static>> {
        let home = fetch_home()?;
        let desktop = fetch_desktop()?;
        let pkg_name = fetch_pkg_name();

        Ok(Environment {
            desktop: Cow::Owned(desktop),
            home: Cow::Owned(home),
            pkg_name: Cow::Owned(pkg_name),
        })
    }

    // --- Getters for idiomatic access ---

    pub fn get_desktop(&self) -> &str {
        &self.desktop
    }

    pub fn get_home(&self) -> &str {
        &self.home
    }

    pub fn get_pkg_name(&self) -> &str {
        &self.pkg_name
    }
}

// --- Private Helper Functions (Internal logic) ---

/// Safely fetches the HOME directory.
fn fetch_home() -> WallSwitchResult<String> {
    env::var("HOME").map_err(|_| WallSwitchError::EnvVarMissing("HOME".to_string()))
}

/// Safely fetches the package name, defaulting to "wallswitch" if not running via Cargo.
fn fetch_pkg_name() -> String {
    env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "wallswitch".to_string())
}

/// Detects the desktop environment by checking common XDG variables.
/// Returns the most descriptive name found or an error if none are present.
fn fetch_desktop() -> WallSwitchResult<String> {
    let mut desktops = Vec::new();

    // Check common environment variables for desktop detection
    for key in [
        "XDG_CURRENT_DESKTOP",
        "XDG_SESSION_DESKTOP",
        "DESKTOP_SESSION",
    ] {
        if let Ok(val) = env::var(key) {
            let val = val.trim().to_lowercase();
            if !val.is_empty() {
                desktops.push(val);
            }
        }
    }

    // Sort by length: usually the longest string contains the most detail
    // (e.g., "ubuntu:gnome" vs just "gnome")
    desktops.sort_by_key(|d| d.len());

    desktops
        .last()
        .cloned()
        .ok_or(WallSwitchError::DesktopDetectionFailed)
}
