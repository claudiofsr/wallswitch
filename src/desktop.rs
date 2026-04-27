use crate::ENVIRON;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

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
            Desktop::Openbox => "openbox",
        };
        write!(f, "{s}")
    }
}

impl Desktop {
    /// Detects the current desktop environment based on the system's global environment state.
    ///
    /// This centralizes the logic for "guessing" which desktop is running. If the environment
    /// variable is missing or unsupported, it defaults to `Openbox`.
    pub fn detect() -> Self {
        // ENVIRON.desktop is typically gathered from $XDG_CURRENT_DESKTOP or $DESKTOP_SESSION.
        Self::from_str(&ENVIRON.desktop).unwrap_or(Desktop::Openbox)
    }
}
