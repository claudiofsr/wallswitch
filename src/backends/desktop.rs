use serde::{Deserialize, Serialize};
use std::{env, fmt, str::FromStr};

/// Table mapping specific environment variables to their respective [`Desktop`] environments.
///
/// Ordered from most specific signatures (e.g., compositor-specific sockets) to most
/// generic fallbacks (e.g., `WAYLAND_DISPLAY`) to ensure accurate detection.
const DETECTION_SIGNATURES: &[(&str, Desktop)] = &[
    ("HYPRLAND_INSTANCE_SIGNATURE", Desktop::Hyprland),
    ("NIRI_SOCKET", Desktop::Niri),
    ("LABWC_PID", Desktop::Labwc),
    ("MANGO_PID", Desktop::Mango),
    ("GNOME_DESKTOP_SESSION_ID", Desktop::Gnome),
    ("GNOME_SHELL_SESSION_MODE", Desktop::Gnome),
    ("XFCE_DESKTOP_SESSION_ID", Desktop::Xfce),
    ("WAYLAND_DISPLAY", Desktop::Wayland),
];

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

/// Implements the `FromStr` trait, allowing us to convert strings into a `Desktop` variant.
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
            Ok(Desktop::Openbox)
        }
    }
}

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
    /// Detects the active desktop environment based on global system state.
    ///
    /// The detection strategy follows two progressive phases:
    /// 1. Queries standard XDG environment variables.
    /// 2. Loops through a static table of specific environment signatures (e.g. compositor sockets,
    ///    display flags) to handle early-boot race conditions where XDG variables are not yet ready.
    ///
    /// If all detection methods fail, it defaults to [`Desktop::Openbox`].
    pub fn detect() -> Self {
        // 1. Try standard XDG environment variables first
        for key in [
            "XDG_CURRENT_DESKTOP",
            "XDG_SESSION_DESKTOP",
            "DESKTOP_SESSION",
        ] {
            if let Ok(val) = env::var(key)
                && let Ok(desktop) = Self::from_str(&val)
                && desktop != Desktop::Openbox
            {
                return desktop;
            }
        }

        // 2. DRY Fallbacks for early autostart race conditions
        for &(var, desktop) in DETECTION_SIGNATURES {
            if env::var(var).is_ok() {
                return desktop;
            }
        }

        Desktop::Openbox
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests_desktop {
    use super::*;

    #[test]
    fn test_from_str_conversions() {
        assert_eq!("GNOME".parse::<Desktop>(), Ok(Desktop::Gnome));
        assert_eq!("Xfce".parse::<Desktop>(), Ok(Desktop::Xfce));
        assert_eq!("ubuntu:gnome".parse::<Desktop>(), Ok(Desktop::Gnome));
        assert_eq!(
            "Hyprland (Wayland)".parse::<Desktop>(),
            Ok(Desktop::Hyprland)
        );
        assert_eq!("random_wm_name".parse::<Desktop>(), Ok(Desktop::Openbox));
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(Desktop::Gnome.to_string(), "gnome");
        assert_eq!(Desktop::Xfce.to_string(), "xfce");
        assert_eq!(Desktop::Hyprland.to_string(), "hyprland");
        assert_eq!(Desktop::Openbox.to_string(), "openbox");
    }

    #[test]
    fn test_serde_serialization_and_deserialization() {
        let serialized = serde_json::to_string(&Desktop::Hyprland).unwrap();
        assert_eq!(serialized, "\"hyprland\"");

        let deserialized: Desktop = serde_json::from_str("\"xfce\"").unwrap();
        assert_eq!(deserialized, Desktop::Xfce);

        let unknown: Desktop = serde_json::from_str("\"unknown_compositor\"").unwrap();
        assert_eq!(unknown, Desktop::Openbox);
    }
}
