use serde::{Deserialize, Serialize};
use std::{env, fmt, str::FromStr};

use crate::is_process_running;

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
    /// Checks if the current session is running under Wayland.
    #[inline]
    pub fn is_wayland(&self) -> bool {
        matches!(
            self,
            Desktop::Hyprland | Desktop::Niri | Desktop::Labwc | Desktop::Mango | Desktop::Wayland
        ) || env::var("WAYLAND_DISPLAY").is_ok()
            || env::var("XDG_SESSION_TYPE")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false)
    }

    /// Detects the active desktop environment with process-table fallback
    /// for robust execution inside systemd user services.
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

        // 2. Specific compositor signatures
        for &(var, desktop) in DETECTION_SIGNATURES {
            if env::var(var).is_ok() {
                return desktop;
            }
        }

        // 3. Process-table fallback (infalível para systemd user services / cron)
        if is_process_running("gnome-shell") {
            return Desktop::Gnome;
        } else if is_process_running("xfce4-session") || is_process_running("xfwm4") {
            return Desktop::Xfce;
        } else if is_process_running("hyprland") {
            return Desktop::Hyprland;
        } else if is_process_running("niri") {
            return Desktop::Niri;
        } else if is_process_running("sway") {
            return Desktop::Wayland;
        }

        // 4. Protocol fallback
        if env::var("WAYLAND_DISPLAY").is_ok()
            || env::var("XDG_SESSION_TYPE")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false)
        {
            return Desktop::Wayland;
        }

        Desktop::Openbox
    }
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

/// cargo test -- --show-output tests_desktop
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
