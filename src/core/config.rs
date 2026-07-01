use crate::{
    Arguments, AtomicWriteExt, Complex, Desktop, Environment, Monitor, Orientation,
    ProceduralEffect, U8Extension, WallSwitchError, WallSwitchResult, get_feh_path, get_monitors,
};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

/// Configurable parameters and custom presets for procedural mathematical overlays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectsConfig {
    /// If true, append custom config presets to the default hardcoded presets.
    ///
    /// If false, use only the presets specified in the config file.
    #[serde(default = "default_true")]
    pub add_presets: bool,
    /// Minimum iteration limit for escape-time fractal calculations.
    #[serde(default = "default_min_iterations")]
    pub min_iterations: u32,
    /// Maximum iteration limit for escape-time fractal calculations.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// User-defined Julia Set presets.
    #[serde(default)]
    pub julia: Vec<CustomFractalPreset>,
    /// User-defined Mandelbrot Set presets.
    #[serde(default)]
    pub mandelbrot: Vec<CustomFractalPreset>,
    /// User-defined Newton-Raphson Basin presets.
    #[serde(default)]
    pub newton: Vec<CustomNewtonPreset>,
    /// User-defined Nova Julia presets.
    #[serde(default)]
    pub nova: Vec<CustomNovaPreset>,
}

impl Default for EffectsConfig {
    /// Initialises default configuration parameters and seeds the configuration with
    /// two distinct presets for each mathematical generator to provide immediate visual variety.
    fn default() -> Self {
        Self {
            add_presets: default_true(),
            min_iterations: default_min_iterations(),
            max_iterations: default_max_iterations(),
            julia: vec![
                CustomFractalPreset {
                    center: Complex { re: -0.8, im: 0.18 },
                    fractal_name: "Stardust spiral galaxy arms".to_string(),
                },
                CustomFractalPreset {
                    center: Complex {
                        re: 0.285,
                        im: 0.535,
                    },
                    fractal_name: "Pinwheel orbital clouds".to_string(),
                },
            ],
            mandelbrot: vec![
                CustomFractalPreset {
                    center: Complex {
                        re: -0.74,
                        im: 0.24,
                    },
                    fractal_name: "custom v1".to_string(),
                },
                CustomFractalPreset {
                    center: Complex {
                        re: -0.088,
                        im: 0.655,
                    },
                    fractal_name: "custom v2".to_string(),
                },
            ],
            newton: vec![
                CustomNewtonPreset {
                    power: 7,
                    lambda: Complex { re: 0.95, im: 0.55 },
                    name: "Aetheric prismatic vortex".to_string(),
                },
                CustomNewtonPreset {
                    power: 3,
                    lambda: Complex { re: 1.50, im: 0.25 },
                    name: "Over-relaxed geometric crown".to_string(),
                },
            ],
            nova: vec![
                CustomNovaPreset {
                    power: 4,
                    c: Complex {
                        re: -0.15,
                        im: -0.35,
                    },
                    r: Complex { re: 1.10, im: 0.20 },
                    name: "Bioluminescent plasma plumes".to_string(),
                },
                CustomNovaPreset {
                    power: 6,
                    c: Complex { re: 0.25, im: 0.40 },
                    r: Complex {
                        re: 0.85,
                        im: -0.15,
                    },
                    name: "Astral jellyfish lattice".to_string(),
                },
            ],
        }
    }
}

/// Helper function providing a default true value for Serde deserialisation.
fn default_true() -> bool {
    true
}

fn default_min_iterations() -> u32 {
    600
}

fn default_max_iterations() -> u32 {
    1200
}

/// A serialized custom Julia/Mandelbrot preset representing the focal point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFractalPreset {
    pub center: Complex,
    pub fractal_name: String,
}

/// A serialized custom Newton preset representing root-finding convergence fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNewtonPreset {
    pub power: u32,
    pub lambda: Complex,
    pub name: String,
}

/// A serialized custom Nova preset representing dynamic fluid-like plumes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNovaPreset {
    pub power: u32,
    pub c: Complex,
    pub r: Complex,
    pub name: String,
}

/// Configuration variables
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Desktops: gnome, xfce, openbox, ...
    pub desktop: Desktop,
    /// Directories containing image files
    pub directories: Vec<PathBuf>,
    /// Image file extension (identify -list format)
    pub extensions: Vec<String>,
    /// Interval (in seconds) between each wallpaper displayed
    pub interval: u64,
    /// Minimum dimension
    pub min_dimension: u64,
    /// Maximum dimension
    pub max_dimension: u64,
    /// Minimum file size
    pub min_size: u64,
    /// Maximum file size
    pub max_size: u64,
    /// Monitor properties
    pub monitors: Vec<Monitor>,
    /// Attach images to monitors in the Horizontal or Vertical orientation
    pub monitor_orientation: Orientation,

    /// Run a single wallpaper update cycle and exit
    #[serde(skip)]
    pub once: bool,

    /// feh binary path
    pub path_feh: PathBuf,
    /// Sort the images found
    pub sort: bool,
    /// Selected procedural overlay effect (none, fractal, star, random)
    pub effect: ProceduralEffect,
    /// Configurable parameters and custom presets for mathematical overlays
    #[serde(default)]
    pub effects: EffectsConfig,
    /// Wallpaper file path used by gnome desktop
    pub wallpaper: PathBuf,

    /// Run without actually applying wallpapers (Simulation mode)
    #[serde(skip)]
    pub dry_run: bool,

    /// Animation transition type for awww daemon
    pub transition_type: String,
    /// Duration of awww transition in seconds
    pub transition_duration: u16,
    /// Framerate of the awww transition animation
    pub transition_fps: u16,
    /// Angle for wipe/wave transitions
    pub transition_angle: u16,
    /// Starting position for center/outer transitions
    pub transition_pos: String,

    /// Show intermediate runtime messages
    #[serde(skip)]
    pub verbose: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::default_with_env(&Environment::fallback())
    }
}

impl Config {
    /// Merges settings from the JSON configuration file with parsed command-line overrides.
    ///
    /// # Errors
    ///
    /// Returns a [`WallSwitchResult`] if config reading or validation fails.
    pub fn new(args: &Arguments, env: &Environment) -> WallSwitchResult<Self> {
        let mut read_default_config = false;
        let config_path = get_config_path(env)?;

        let config: Config = match read_config_file(&config_path) {
            Ok(configuration) => configuration,
            Err(_) => {
                read_default_config = true;
                Self::default_with_env(env)
            }
        }
        .set_command_line_arguments(args)?
        .validate_config()?
        .write_config_file(&config_path, read_default_config)?;

        Ok(config)
    }

    /// Initializes a default configuration aligned with the active environment context.
    pub fn default_with_env(env: &Environment) -> Self {
        // Set image extensions (identify -list format)
        let extensions: Vec<String> = ["avif", "jpg", "jpeg", "png", "tif", "webp"]
            .iter()
            .map(ToString::to_string)
            .collect();

        // Interval: 30 * 60 = 1800 seconds (30 minutes)
        let interval: u64 = 30 * 60;

        // Dimension.height >= min_dimension && dimension.width >= min_dimension
        let min_dimension: u64 = 600;

        // Dimension.height <= max_dimension && dimension.width <= max_dimension
        let max_dimension: u64 = 128_000;

        Config {
            desktop: Desktop::detect(),
            min_dimension,
            max_dimension,
            min_size: u64::pow(1024, 1),
            max_size: u64::pow(1024, 3),
            directories: get_directories(env).unwrap_or_default(),
            effect: ProceduralEffect::None,
            effects: EffectsConfig::default(),
            extensions,
            interval,
            monitors: get_monitors(2),
            monitor_orientation: Orientation::Horizontal,
            once: false,
            path_feh: PathBuf::from("/usr/bin/feh"),
            sort: false,
            wallpaper: get_wallpaper_path(env).unwrap_or_default(),
            dry_run: false,
            transition_type: "random".to_string(),
            transition_duration: 2,
            transition_fps: 60,
            transition_angle: 45,
            transition_pos: "center".to_string(),
            verbose: false,
        }
    }
}

/// Resolves the absolute path to the configuration JSON file.
///
/// Config file path: "/home/user_name/.config/wallswitch/wallswitch.json"
pub fn get_config_path(env: &Environment) -> WallSwitchResult<PathBuf> {
    let mut config_path = env.get_app_config_dir();
    config_path.push(env.get_pkg_name());
    config_path.set_extension("json");
    Ok(config_path)
}

/// Resolves the standard wallpaper output file path under the user's cache directory.
///
/// This resolves to `~/.cache/wallswitch/wallswitch.png` on Unix-like systems and
/// `%LOCALAPPDATA%\wallswitch\wallswitch.png` on Windows.
///
/// Saving the final compiled wallpaper within the user's cache partition is the standard
/// best practice under the XDG Base Directory Specification. This prevents heavy, volatile
/// binary image files from cluttering the user's configuration directory (`~/.config`),
/// which is often targeted by system backups or dotfiles version control (Git).
///
/// # Errors
///
/// Returns a [`WallSwitchResult`] if the parent environment path cannot be resolved.
pub fn get_wallpaper_path(env: &Environment) -> WallSwitchResult<PathBuf> {
    let mut wallpaper_path = env.get_app_cache_dir();

    // Appends "wallswitch.png" inside the app's user cache directory
    wallpaper_path.push(env.get_pkg_name());
    wallpaper_path.set_extension("png");

    Ok(wallpaper_path)
}

/// Discovers standard directories where user wallpapers are located.
pub fn get_directories(env: &Environment) -> WallSwitchResult<Vec<PathBuf>> {
    let home_dir = env.get_home_dir();
    let images = ["Figures", "Images", "Pictures", "Wallpapers", "Imagens"];

    let mut directories: Vec<PathBuf> = images.iter().map(|image| home_dir.join(image)).collect();

    let unix_candidates = [
        PathBuf::from("/usr/share/wallpapers"),
        PathBuf::from("/usr/share/backgrounds"),
    ];

    for path in unix_candidates {
        if path.exists() {
            directories.push(path);
        }
    }

    if let Some(system_root) = env::var_os("SystemRoot") {
        let win_wallpaper = Path::new(&system_root).join("Web").join("Wallpaper");
        if win_wallpaper.exists() {
            directories.push(win_wallpaper);
        }
    }

    Ok(directories)
}

// Set boundary config values
fn config_boundary() -> Config {
    Config {
        interval: 5,
        min_dimension: 10,
        min_size: 1,
        monitors: vec![Monitor::default()],
        ..Config::default()
    }
}

impl Config {
    /// Check if the value is in the range
    pub fn in_range(&self, value: u64) -> bool {
        self.min_dimension <= value && value <= self.max_dimension
    }

    /// Print Config
    pub fn print(&self) -> WallSwitchResult<()> {
        let json: String = serde_json::to_string_pretty(self)?;
        println!("Config:\n{json}\n");

        Ok(())
    }

    /// Set command-line arguments for configuration
    ///
    /// Update self: Config values
    fn set_command_line_arguments(mut self, args: &Arguments) -> WallSwitchResult<Self> {
        if let Some(min_dimension) = args.min_dimension {
            self.min_dimension = min_dimension;
        }

        if let Some(max_dimension) = args.max_dimension {
            self.max_dimension = max_dimension;
        }

        if let Some(min_size) = args.min_size {
            self.min_size = min_size;
        }

        if let Some(max_size) = args.max_size {
            self.max_size = max_size;
        }

        if let Some(interval) = args.interval {
            self.interval = interval;
        }

        if let Some(monitor) = args.monitor {
            self.monitors = get_monitors(monitor.into());
        }

        if let Some(orientation) = &args.monitor_orientation {
            self.monitor_orientation = orientation.clone();
        }

        if let Some(pictures_per_monitor) = args.pictures_per_monitor {
            for monitor in &mut self.monitors {
                monitor.pictures_per_monitor = pictures_per_monitor;
            }
        }

        self.once = args.once;

        if args.dry_run {
            self.dry_run = true;
            self.once = true; // Force a single execution cycle and exit on dry-run
        }
        if let Some(ref t) = args.transition_type {
            self.transition_type = t.clone();
        }
        if let Some(d) = args.transition_duration {
            self.transition_duration = d;
        }
        if let Some(f) = args.transition_fps {
            self.transition_fps = f;
        }
        if let Some(a) = args.transition_angle {
            self.transition_angle = a;
        }
        if let Some(ref p) = args.transition_pos {
            self.transition_pos = p.clone();
        }

        if let Some(effect) = args.effect {
            self.effect = effect;
        }

        // Apply CLI overrides for procedural mathematical overlay details (EffectsConfig)
        if let Some(effects_add_presets) = args.effects_add_presets {
            self.effects.add_presets = effects_add_presets;
        }

        if let Some(effects_min_iterations) = args.effects_min_iterations {
            self.effects.min_iterations = effects_min_iterations;
        }

        if let Some(effects_max_iterations) = args.effects_max_iterations {
            self.effects.max_iterations = effects_max_iterations;
        }

        if args.sort {
            self.sort = !self.sort;
        }

        if args.verbose {
            self.verbose = !self.verbose;
        }

        self.desktop = Desktop::detect(); // Update desktop

        Ok(self)
    }

    /// Validate configuration
    pub fn validate_config(mut self) -> WallSwitchResult<Self> {
        let boundary: Config = config_boundary();

        // Note: Multiple pictures per monitor (-p) is now fully supported on all desktops!

        if self.interval < boundary.interval {
            let value = self.interval.to_string();
            return Err(WallSwitchError::AtLeastValue {
                arg: "--interval".to_string(),
                value,
                num: boundary.interval,
            });
        }

        if !self.path_feh.is_file() {
            self.path_feh = get_feh_path(true)?;
        }

        if self.min_dimension < boundary.min_dimension {
            let value = self.min_dimension.to_string();
            return Err(WallSwitchError::AtLeastValue {
                arg: "--min_dimension".to_string(),
                value,
                num: boundary.min_dimension,
            });
        }

        if self.min_size < boundary.min_size {
            let value = self.min_size.to_string();
            return Err(WallSwitchError::AtLeastValue {
                arg: "--min_size".to_string(),
                value,
                num: boundary.min_size,
            });
        }

        if self.monitors.is_empty() {
            let value = self.monitors.len().to_string();
            return Err(WallSwitchError::AtLeastValue {
                arg: "--interval".to_string(),
                value,
                num: 1,
            });
        }

        for monitor in &self.monitors {
            if monitor.pictures_per_monitor < 1 {
                let value = monitor.pictures_per_monitor.to_string();
                return Err(WallSwitchError::AtLeastValue {
                    arg: "--picture".to_string(),
                    value,
                    num: 1,
                });
            }
        }

        if let Some(parent) = self.wallpaper.parent()
            && !parent.exists()
        {
            // Ensure the directory exists
            fs::create_dir_all(parent)?;
        }

        // Validate basic boundary pairs
        for (min, max) in [
            (self.min_dimension, self.max_dimension),
            (self.min_size, self.max_size),
        ] {
            if min > max {
                return Err(WallSwitchError::MinMax { min, max });
            }
        }

        // Validate that min_iterations does not exceed max_iterations
        if self.effects.min_iterations > self.effects.max_iterations {
            return Err(WallSwitchError::MinMax {
                min: self.effects.min_iterations as u64,
                max: self.effects.max_iterations as u64,
            });
        }

        Ok(self)
    }

    /// Write config file path:: "/home/user_name/.config/wallswitch/wallswitch.json"
    ///
    /// To ensure atomic write and protect against sudden crashes or interruptions,
    /// the configuration is written to a temporary file in the target directory
    /// and then renamed.
    pub fn write_config_file(
        self,
        path: &Path,
        read_default_config: bool,
    ) -> WallSwitchResult<Self> {
        if read_default_config {
            eprintln!("Create the configuration file: {path:?}\n");
        }

        path.atomic_write(|temp_path| {
            let file = File::create(temp_path).map_err(|io_error| WallSwitchError::IOError {
                path: temp_path.to_path_buf(),
                io_error,
            })?;
            let mut writer = BufWriter::new(file);
            serde_json::to_writer_pretty(&mut writer, &self)?;
            writer.flush()?;
            Ok(())
        })?;

        Ok(self)
    }

    /// Get the number of images per cycle
    pub fn get_number_of_images(&self) -> usize {
        self.monitors
            .iter()
            .map(|monitor| monitor.pictures_per_monitor.to_usize())
            .sum()
    }
}

/// Read config file path: "/home/user_name/.config/wallswitch/wallswitch.json"
pub fn read_config_file<P>(path: P) -> WallSwitchResult<Config>
where
    P: AsRef<Path>,
{
    // Open the file in read-only mode with buffer
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Config`
    let config: Config = serde_json::from_reader(reader)?;

    Ok(config)
}

//----------------------------------------------------------------------------//
//                                   Tests                                    //
//----------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_path() -> WallSwitchResult<()> {
        let env = Environment::new()?;
        let path = get_config_path(&env)?;
        assert!(path.to_string_lossy().contains(".config"));
        assert!(path.to_string_lossy().ends_with("wallswitch.json"));
        Ok(())
    }

    #[test]
    fn test_atomic_write_config_file() {
        let env = Environment::new().unwrap();
        let target_dir = env.get_temp_dir().join("wall_test");
        let target_file = target_dir.join("config.json");

        let config = Config::default();
        let write_result = config.write_config_file(&target_file, false);
        assert!(write_result.is_ok());
        assert!(target_file.exists());

        let read_back = read_config_file(&target_file);
        assert!(read_back.is_ok());
        assert_eq!(read_back.unwrap().interval, 1800);

        let _ = fs::remove_dir_all(target_dir);
    }
}
