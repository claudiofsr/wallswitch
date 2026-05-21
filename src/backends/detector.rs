use crate::{Config, Desktop, WallSwitchError, WallSwitchResult, exec_cmd, is_installed};
use std::{fs, path::PathBuf, process::Command};

/// Detects active outputs (monitors) using a robust fallback chain.
/// Returns a WallSwitchResult wrapping a Vector of monitor property strings.
///
/// If all detection methods fail, it returns a formal WallSwitchError instead of
/// relying on hardcoded defaults.
pub fn detect_monitors(config: &Config) -> WallSwitchResult<Vec<String>> {
    let mut monitors = Vec::new();

    // 1. Try Desktop-specific tools first
    match config.desktop {
        Desktop::Niri if is_installed("niri") => {
            if let Ok(out) = Command::new("niri").args(["msg", "outputs"]).output() {
                monitors = parse_niri(&String::from_utf8_lossy(&out.stdout));
            }
        }
        Desktop::Hyprland if is_installed("hyprctl") => {
            if let Ok(out) = Command::new("hyprctl").arg("monitors").output() {
                monitors = parse_hyprland(&String::from_utf8_lossy(&out.stdout));
            }
        }
        Desktop::Xfce if is_installed("xfconf-query") => {
            let active_xrandr_monitors = get_active_xrandr_monitors(config);

            // Self-healing: remove orphan properties before detection
            prune_stale_xfce_configs(config, &active_xrandr_monitors)?;

            let mut cmd = Command::new("xfconf-query");
            cmd.args([
                "--channel",
                "xfce4-desktop",
                "--property",
                "/backdrop",
                "--list",
            ]);
            if let Ok(out) = exec_cmd(&mut cmd, config.verbose, "xfconf-query") {
                monitors = parse_xfce(
                    &String::from_utf8_lossy(&out.stdout),
                    &active_xrandr_monitors,
                );
            }
        }
        _ => {}
    }

    if !monitors.is_empty() {
        return Ok(monitors);
    }

    // 2. Generic Wayland fallback (wlr-randr)
    if is_installed("wlr-randr")
        && let Ok(out) = Command::new("wlr-randr").output()
    {
        monitors = parse_wlr_randr(&String::from_utf8_lossy(&out.stdout));
        if !monitors.is_empty() {
            return Ok(monitors);
        }
    }

    // 3. Hardware fallback (DRM Sysfs - Linux Kernel)
    monitors = detect_drm_monitors();
    if !monitors.is_empty() {
        return Ok(monitors);
    }

    // 4. Fatal Error: If we reached this point, no active monitors were found.
    // Instead of using a fake default, we raise a NoMonitors error.
    Err(WallSwitchError::NoMonitors(
        "any system tool (X11/Wayland/DRM)".to_string(),
    ))
}

/// Get active X11 monitors via xrandr to filter out stale configurations.
///
/// Runs `xrandr --listactivemonitors` and safely parses the output.
pub fn get_active_xrandr_monitors(config: &Config) -> Vec<String> {
    let mut monitors = Vec::new();

    if is_installed("xrandr") {
        let mut cmd = Command::new("xrandr");
        cmd.args(["--listactivemonitors"]);
        if let Ok(out) = exec_cmd(&mut cmd, config.verbose, "xrandr") {
            let stdout = String::from_utf8_lossy(&out.stdout);
            monitors = parse_xrandr(&stdout);
        }
    }

    monitors
}

/*
┌─[claudio@manjaro] - [~/Documents/Rust/projects/wallswitch] - [seg mai 18, 10:16]
└─[$] <git:(master*)> xrandr --listactivemonitors
Monitors: 2
 0: +DP-2 3840/621x2160/341+0+0  DP-2
 1: +DP-0 3840/621x2160/341+3840+0  DP-0
┌─[claudio@manjaro] - [~/Documents/Rust/projects/wallswitch] - [seg mai 18, 10:16]
└─[$] <git:(master*)>
*/

pub fn parse_xrandr(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() >= 3 {
                let first_token = tokens[0];
                if let Some(prefix) = first_token.strip_suffix(':')
                    && prefix.parse::<usize>().is_ok()
                {
                    return tokens.last().map(|&s| s.to_string());
                }
            }
            None
        })
        .collect()
}

/// XFCE Logic: Matches active hardware (xrandr) with XFCE properties.
/// If a monitor is active but has no XFCE property yet, we synthesize one.
pub fn parse_xfce(stdout: &str, active_monitors: &[String]) -> Vec<String> {
    let words = ["screen0", "workspace0", "last-image"];
    let existing_props: Vec<String> = stdout
        .trim()
        .split(['\n', ' '])
        .filter(|out| words.iter().all(|w| out.contains(w)))
        .map(String::from)
        .collect();

    if active_monitors.is_empty() {
        return existing_props;
    }

    let mut final_properties = Vec::new();

    for m in active_monitors {
        let prefix_match = format!("/monitor{}/", m);
        let exact_match = format!("/{}/", m);

        // Check if XFCE already has a property for this monitor
        if let Some(prop) = existing_props.iter().find(|p| p.contains(&prefix_match)) {
            final_properties.push(prop.clone());
        } else if let Some(prop) = existing_props.iter().find(|p| p.contains(&exact_match)) {
            final_properties.push(prop.clone());
        } else {
            // CRITICAL FIX: If monitor exists in xrandr but NOT in xfconf,
            // we create the expected path based on XFCE 4.18+ standards.
            // This allows the program to "force-set" the wallpaper on new monitors.
            let synthesized = format!("/backdrop/screen0/monitor{}/workspace0/last-image", m);
            final_properties.push(synthesized);
        }
    }

    // Deduplicate in case multiple matches occurred
    final_properties.sort();
    final_properties.dedup();
    final_properties
}

pub fn prune_stale_xfce_configs(
    config: &Config,
    active_monitors: &[String],
) -> WallSwitchResult<()> {
    if active_monitors.is_empty() {
        return Ok(());
    }

    let mut cmd = Command::new("xfconf-query");
    cmd.args([
        "--channel",
        "xfce4-desktop",
        "--property",
        "/backdrop",
        "--list",
    ]);

    if let Ok(output) = cmd.output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stale_properties: Vec<String> = stdout
            .lines()
            .filter(|prop| prop.contains("workspace0/last-image"))
            .filter(|prop| {
                !active_monitors.iter().any(|m| {
                    prop.contains(&format!("/{}/", m)) || prop.contains(&format!("/monitor{}/", m))
                })
            })
            .map(String::from)
            .collect();

        for property in stale_properties {
            if let Some(monitor_root) = property.split("/workspace0").next() {
                if config.verbose {
                    println!("Pruning: {}", monitor_root);
                }
                let _ = Command::new("xfconf-query")
                    .args([
                        "--channel",
                        "xfce4-desktop",
                        "--property",
                        monitor_root,
                        "--reset",
                        "--recursive",
                    ])
                    .output();
            }
        }
    }
    Ok(())
}

/// Pure parser for Niri output
pub fn parse_niri(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.starts_with("Output"))
        .filter_map(|line| {
            let start = line.rfind('(')?;
            let end = line.rfind(')')?;
            if start < end {
                Some(line[start + 1..end].to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Pure parser for Hyprland output
pub fn parse_hyprland(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| line.starts_with("Monitor"))
        .filter_map(|line| line.split_whitespace().nth(1).map(String::from))
        .collect()
}

/// Pure parser for wlr-randr output
pub fn parse_wlr_randr(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|line| !line.starts_with(' ') && !line.is_empty())
        .filter_map(|line| line.split_whitespace().next().map(String::from))
        .collect()
}

/// Hardware DRM parser
fn detect_drm_monitors() -> Vec<String> {
    let mut monitors = Vec::new();
    let drm_path = PathBuf::from("/sys/class/drm");

    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") && name.contains('-') {
                let status_path = entry.path().join("status");
                if let Ok(status) = fs::read_to_string(status_path)
                    && status.trim() == "connected"
                    && let Some(idx) = name.find('-')
                {
                    monitors.push(name[idx + 1..].to_string());
                }
            }
        }
    }
    monitors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_parsers() {
        let expected = vec!["DP-1", "DP-2"];

        // Mocked xrandr output with malicious/unexpected edge cases
        let xrandr_mock = "\
Monitors: 2
 0: +DP-1 3840/621x2160/341+0+0  DP-1
 Error: bla bla
 A: +DP-2 3840/621x2160/341+0+0  DP-X
 1: +DP-2 3840/621x2160/341+3840+0  DP-2";
        assert_eq!(parse_xrandr(xrandr_mock), expected);

        // Mocked Niri output
        let niri_mock = "\
Output eDP-1 (DP-1)
  Mode: 1920x1080
Output HDMI-A-1 (DP-2)
  Mode: 1920x1080";
        assert_eq!(parse_niri(niri_mock), expected);

        // Mocked Hyprland output
        let hypr_mock = "\
Monitor DP-1 (ID 0):
  1920x1080@60.00000
Monitor DP-2 (ID 1):
  1920x1080@60.00000";
        assert_eq!(parse_hyprland(hypr_mock), expected);

        // Mocked wlr-randr output
        let wlr_mock = "\
DP-1 \"Manufacturer X\"
  Position: 0,0
DP-2 \"Manufacturer Y\"
  Position: 1920,0";
        assert_eq!(parse_wlr_randr(wlr_mock), expected);

        // Mocked XFCE output with stale/duplicate entries
        let xfce_mock = "\
/backdrop/screen0/monitorASUSPB287Q/workspace0/last-image
/backdrop/screen0/monitorDP-1/workspace0/last-image
/backdrop/screen0/DP-2/workspace0/last-image
/backdrop/screen0/monitorDP-2/workspace0/last-image
/backdrop/screen0/monitorDP-1/workspace0/color-style";

        let active = vec!["DP-1".to_string(), "DP-2".to_string()];

        // It MUST prioritize prefix_match (/monitorDP-2/) over exact_match (/DP-2/)
        let xfce_expected = vec![
            "/backdrop/screen0/monitorDP-1/workspace0/last-image",
            "/backdrop/screen0/monitorDP-2/workspace0/last-image",
        ];
        assert_eq!(parse_xfce(xfce_mock, &active), xfce_expected);

        // Test fallback if xrandr doesn't match anything
        let empty_active = vec![];
        let xfce_expected_fallback = vec![
            "/backdrop/screen0/monitorASUSPB287Q/workspace0/last-image",
            "/backdrop/screen0/monitorDP-1/workspace0/last-image",
            "/backdrop/screen0/DP-2/workspace0/last-image",
            "/backdrop/screen0/monitorDP-2/workspace0/last-image",
        ];
        assert_eq!(parse_xfce(xfce_mock, &empty_active), xfce_expected_fallback);
    }
}
