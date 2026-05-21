use crate::{
    Arguments, Desktop, Environment, Monitor, Orientation, ProceduralEffect, U8Extension,
    WallSwitchError, WallSwitchResult, get_feh_path, get_monitors,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

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
            min_size: u64::pow(1024, 1), // 1024 ^ 1 = 1kb
            max_size: u64::pow(1024, 3), // 1024 ^ 3 = 1Gb
            directories: get_directories().unwrap_or_default(),
            effect: ProceduralEffect::None,
            extensions,
            interval,
            monitors: get_monitors(2),
            monitor_orientation: Orientation::Horizontal,
            once: false,
            path_feh: PathBuf::from("/usr/bin/feh"),
            sort: false,
            verbose: false,
            wallpaper: get_wallpaper_path().unwrap_or_default(),
            dry_run: false,
            transition_type: "random".to_string(),
            transition_duration: 2,
            transition_fps: 60,
            transition_angle: 45,
            transition_pos: "center".to_string(),
        }
    }
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
    /// Merges settings from the JSON file with parsed command-line arguments.
    ///
    /// Priority: 1. CLI Args -> 2. Config File -> 3. Defaults.
    pub fn new(args: &Arguments) -> WallSwitchResult<Self> {
        let mut read_default_config = false;
        let config_path: PathBuf = get_config_path()?;

        // Attempt to read the existing JSON file; fallback to Default if missing
        let config: Config = match read_config_file(&config_path) {
            Ok(configuration) => configuration,
            Err(_) => {
                read_default_config = true;
                Self::default()
            }
        }
        // Apply CLI overrides, validate values, and sync the JSON file back to disk
        .set_command_line_arguments(args)?
        .validate_config()?
        .write_config_file(&config_path, read_default_config)?;

        Ok(config)
    }

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
            self.monitors = get_monitors(monitor.into())
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

        for (min, max) in [
            (self.min_dimension, self.max_dimension),
            (self.min_size, self.max_size),
        ] {
            if min > max {
                return Err(WallSwitchError::MinMax { min, max });
            }
        }

        Ok(self)
    }

    /// Write config file path:: "/home/user_name/.config/wallswitch/wallswitch.json"
    pub fn write_config_file(
        self,
        path: &PathBuf,
        read_default_config: bool,
    ) -> WallSwitchResult<Self> {
        if read_default_config {
            eprintln!("Create the configuration file: {path:?}\n");
        }

        // Recursively create a directory and all of its parent components if they are missing
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?
        };

        let file: File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|io_error| WallSwitchError::IOError {
                path: path.to_path_buf(),
                io_error,
            })?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self)?;
        writer.flush()?;

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

/// Default wallpaper path: "/home/user_name/wallswitch.jpg"
pub fn get_wallpaper_path() -> WallSwitchResult<PathBuf> {
    let env = Environment::new()?;
    let home = env.get_home();
    let pkg_name = env.get_pkg_name();

    let mut wallpaper_path: PathBuf = [home, pkg_name].iter().collect();
    wallpaper_path.set_extension("jpg");

    Ok(wallpaper_path)
}

/// Default directories to search for images
pub fn get_directories() -> WallSwitchResult<Vec<PathBuf>> {
    let env = Environment::new()?;
    let home = env.get_home();

    let images = ["Figures", "Images", "Pictures", "Wallpapers", "Imagens"];

    // Create a vector of image directories under the home directory
    let directories_home: Vec<PathBuf> = images
        .into_iter()
        .map(|image| Path::new(home).join(image))
        .collect();

    let sep: &str = std::path::MAIN_SEPARATOR_STR;

    // Add default system backgrounds directories
    let path1: PathBuf = [sep, "usr", "share", "wallpapers"].iter().collect();
    let path2: PathBuf = [sep, "usr", "share", "backgrounds"].iter().collect();
    let path3: PathBuf = [sep, "tmp", "teste"].iter().collect();

    // Create a vector of additional image directories
    let directories_others: Vec<PathBuf> = vec![path1, path2, path3];

    // Combine the two vectors and return
    Ok(directories_home
        .into_iter()
        .chain(directories_others)
        .collect())
}

/// Config file path: "/home/user_name/.config/wallswitch/wallswitch.json"
pub fn get_config_path() -> WallSwitchResult<PathBuf> {
    let env = Environment::new()?;
    let home = env.get_home();
    let pkg_name = env.get_pkg_name();
    let hidden_dir = ".config";

    let mut config_path: PathBuf = [home, hidden_dir, pkg_name, pkg_name].iter().collect();
    config_path.set_extension("json");

    Ok(config_path)
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
