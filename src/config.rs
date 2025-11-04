use crate::{
    Arguments, ENVIRON, Monitor, Orientation, U8Extension, WallSwitchError, WallSwitchResult,
    get_feh_path, get_magick_path, get_monitors,
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
    pub desktop: String,
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
    /// feh binary path
    pub path_feh: PathBuf,
    /// magick binary path
    pub path_magick: PathBuf,
    /// Sort the images found.
    pub sort: bool,
    /// Show intermediate runtime messages
    pub verbose: bool,
    /// Wallpaper file path used by gnome desktop
    pub wallpaper: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        // set image extensions
        // identify -list format
        let extensions: Vec<String> = ["avif", "jpg", "jpeg", "png", "svg", "tif", "webp"]
            .iter()
            .map(ToString::to_string)
            .collect();

        // interval: 30 * 60 = 1800 seconds (30 minutes)
        let interval: u64 = 30 * 60;

        // dimension.height >= min_dimension && dimension.width >= min_dimension
        let min_dimension: u64 = 600;

        // dimension.height <= max_dimension && dimension.width <= max_dimension
        let max_dimension: u64 = 128_000;

        Config {
            desktop: ENVIRON.desktop.to_string(),
            min_dimension,
            max_dimension,
            min_size: u64::pow(1024, 1), // 1024 ^ 1 = 1kb
            max_size: u64::pow(1024, 3), // 1024 ^ 3 = 1Gb
            directories: get_directories(),
            extensions,
            interval,
            monitors: get_monitors(2),
            monitor_orientation: Orientation::Horizontal,
            path_feh: PathBuf::from("/usr/bin/feh"),
            path_magick: PathBuf::from("/usr/bin/magick"),
            sort: false,
            verbose: false,
            wallpaper: get_wallpaper_path(),
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
    /// Read command line arguments with priority order:
    ///
    /// 1. read config file || read default config
    /// 2. set_command_line_arguments
    /// 3. validate_config
    /// 4. write_config_file
    ///
    /// At the end add or update config file.
    pub fn new() -> WallSwitchResult<Self> {
        let mut read_default_config = false;
        let config_path: PathBuf = get_config_path()?;
        let args = Arguments::build()?;

        let config: Config = match read_config_file(&config_path) {
            Ok(configuration) => configuration,
            Err(_) => {
                read_default_config = true;
                Self::default()
            }
        }
        .set_command_line_arguments(&args)?
        .validate_config()?
        .write_config_file(&config_path, read_default_config)?;

        Ok(config)
    }

    /// Check if the value is in the range.
    pub fn in_range(&self, value: u64) -> bool {
        self.min_dimension <= value && value <= self.max_dimension
    }

    /// Print Config.
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

        if args.sort {
            self.sort = !self.sort;
        }

        if args.verbose {
            self.verbose = !self.verbose;
        }

        self.desktop = ENVIRON.desktop.to_string(); // update desktop

        Ok(self)
    }

    /// Validate configuration
    pub fn validate_config(mut self) -> WallSwitchResult<Self> {
        let boundary: Config = config_boundary();

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

        if !self.path_magick.is_file() {
            self.path_magick = get_magick_path(true)?;
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
                arg: "--interval".to_string(), // This looks like it should be --monitors or similar
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
            fs::create_dir_all(parent)?; // This works due to #[from] io::Error
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

        // Recursively create a directory and all of its parent components if they are missing.
        if let Some(parent) = path.parent() {
            // println!("parent: {parent:?}");
            fs::create_dir_all(parent)?
        };

        //let file = File::create(path)?;

        let file: File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|io_error| {
                // Add a custom error message
                WallSwitchError::IOError {
                    path: path.to_path_buf(),
                    io_error,
                }
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

/// default wallpaper path: "/home/user_name/wallswitch.jpg"
pub fn get_wallpaper_path() -> PathBuf {
    let home = ENVIRON.get_home();
    let pkg_name = ENVIRON.get_pkg_name();

    let mut wallpaper_path: PathBuf = [home, pkg_name].iter().collect();
    wallpaper_path.set_extension("jpg");

    wallpaper_path
}

/// Default directories to search for images.
pub fn get_directories() -> Vec<PathBuf> {
    /*
    "/home/user_name/Figures",
    "/home/user_name/Images",
    "/home/user_name/Pictures",
    "/home/user_name/Wallpapers",
    "/home/user_name/Imagens",
    */

    let home = ENVIRON.get_home(); // "/home/user_name"
    let images = ["Figures", "Images", "Pictures", "Wallpapers", "Imagens"];

    // Create a vector of image directories under the home directory
    let directories_home: Vec<PathBuf> = images
        .into_iter()
        .map(|image| Path::new(home).join(image)) // add "home/image"
        .collect();

    /*
    "/usr/share/wallpapers",
    "/usr/share/backgrounds",
    "/tmp/teste",
    */

    let sep: &str = std::path::MAIN_SEPARATOR_STR;

    // add "/usr/share/wallpapers"
    let path1: PathBuf = [sep, "usr", "share", "wallpapers"].iter().collect();
    let path2: PathBuf = [sep, "usr", "share", "backgrounds"].iter().collect();
    let path3: PathBuf = [sep, "tmp", "teste"].iter().collect();

    // Create a vector of additional image directories
    let directories_others: Vec<PathBuf> = vec![path1, path2, path3];

    // Combine the two vectors and return
    directories_home
        .into_iter()
        .chain(directories_others)
        .collect()
}

/// Config file path: "/home/user_name/.config/wallswitch/wallswitch.json"
pub fn get_config_path() -> WallSwitchResult<PathBuf> {
    let home = ENVIRON.get_home();
    let hidden_dir = ".config";
    let pkg_name = ENVIRON.get_pkg_name();

    let mut config_path: PathBuf = [home, hidden_dir, pkg_name, pkg_name].iter().collect();
    config_path.set_extension("json");

    Ok(config_path)
}

/// Read config file path: "/home/user_name/.config/wallswitch/wallswitch.json"
pub fn read_config_file<P>(path: P) -> WallSwitchResult<Config>
where
    P: AsRef<Path>,
{
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Config`.
    let config: Config = serde_json::from_reader(reader)?;

    Ok(config)
}
