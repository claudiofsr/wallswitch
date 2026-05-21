use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

use crate::Environment;

/// Represents the supported Desktop Environments (DE) or Window Managers (WM).
/// We use an Enum instead of a String to ensure type safety and prevent logic errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// This attribute tells Serde to serialize/deserialize variants as lowercase strings (e.g., "gnome").
#[serde(rename_all = "lowercase")]
pub enum Desktop {
    Gnome,
    Xfce,
    Hyprland,
    Niri,
    Labwc,   // New: Lightweight Wayland compositor
    Mango,   // New: Mango WM
    Wayland, // New: Generic fallback for Wayland environments
    /// The `#[serde(other)]` attribute is a fallback. If the JSON configuration file
    /// contains a value not listed above, Serde will automatically map it to `Openbox`.
    #[serde(other)]
    Openbox,
}

/// Implements the `FromStr` trait, allowing us to convert strings (like environment variables)
/// into a `Desktop` variant using `s.parse::<Desktop>()`.
impl FromStr for Desktop {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We convert to lowercase to make the detection case-insensitive.
        let s = s.to_lowercase();

        // We use .contains() because session variables often look like "ubuntu:gnome"
        // or "Hyprland (Wayland)". This makes the detection more flexible.
        if s.contains("gnome") {
            Ok(Desktop::Gnome)
        } else if s.contains("xfce") {
            Ok(Desktop::Xfce)
        } else if s.contains("hyprland") {
            Ok(Desktop::Hyprland)
        } else if s.contains("niri") {
            Ok(Desktop::Niri)
        } else if s.contains("labwc") {
            Ok(Desktop::Labwc)
        } else if s.contains("mango") {
            Ok(Desktop::Mango)
        } else if s.contains("wayland") {
            Ok(Desktop::Wayland)
        } else {
            // Default fallback for any unknown environment.
            Ok(Desktop::Openbox)
        }
    }
}

/// Implements the `Display` trait so we can easily print the enum or convert it back
/// to a string using `.to_string()`.
impl fmt::Display for Desktop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Desktop::Gnome => "gnome",
            Desktop::Xfce => "xfce",
            Desktop::Hyprland => "hyprland",
            Desktop::Niri => "niri",
            Desktop::Labwc => "labwc",
            Desktop::Mango => "mango",
            Desktop::Wayland => "wayland",
            Desktop::Openbox => "openbox",
        };
        write!(f, "{s}")
    }
}

impl Desktop {
    /// Detects the current desktop environment based on the system's global environment state.
    ///
    /// This centralizes the logic for "guessing" which desktop is running.
    /// If the environment variables are missing or cannot be read, it safely
    /// defaults to `Openbox` instead of panicking.
    pub fn detect() -> Self {
        // We call Environment::new() and handle the result.
        // If successful, we parse the desktop string; otherwise, we use the fallback.
        Environment::new()
            .map(|env| Self::from_str(&env.desktop).unwrap_or(Desktop::Openbox))
            .unwrap_or(Desktop::Openbox)
    }
}
