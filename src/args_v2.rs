// use args_v2
// cargo b -r && cargo install --path=. --features args_v2

use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    str::{self, FromStr},
};

use crate::{
    ENVIRON, Orientation,
    WallSwitchError::{self, *},
    WallSwitchResult, get_config_path, read_config_file,
};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments {
    // see config::config_boundary() values
    /// Set a minimum file size (in bytes) for searching image files.
    ///
    /// keep files whose size is greater than or equal to a minimum value.
    ///
    /// size >= min_size
    pub min_size: Option<u64>,

    /// Set a maximum file size (in bytes) for searching image files.
    ///
    /// keep files whose size is less than or equal to a maximum value.
    ///
    /// size <= max_size
    pub max_size: Option<u64>,

    /// Read the configuration file and exit the program.
    pub config: bool,

    /// Set the minimum dimension that the height and width must satisfy.
    ///
    /// width >= min_dimension && height >= min_dimension
    ///
    /// Default value: min_dimension = 600.
    pub min_dimension: Option<u64>,

    /// Set the maximum dimension that the height and width must satisfy.
    ///
    /// width <= max_dimension && height <= max_dimension
    pub max_dimension: Option<u64>,

    /// Print help (see more with '--help')
    pub help: bool,

    /// Set the interval (in seconds) between each wallpaper displayed.
    ///
    /// Default value: interval = 30 * 60 = 1800 seconds (30 minutes).
    pub interval: Option<u64>,

    /// Set the number of monitors [default: 2]
    pub monitor: Option<u8>,

    /// Inform monitor orientation: Horizontal (side-by-side) or Vertical (stacked).
    ///
    /// Orientation: [Horizontal, Vertical]
    ///
    /// Default orientation: Horizontal.
    pub monitor_orientation: Option<Orientation>,

    /// Set number of pictures (or images) per monitor [default: 1]
    ///
    /// Each monitor can have a diferent number of pictures (or images)
    ///
    /// Gnome desktop only
    pub pictures_per_monitor: Option<u8>,

    /// Sort the images found.
    pub sort: bool,

    /// Show intermediate runtime messages.
    ///
    /// Show found images.
    ///
    /// Show pid numbers of previous running program.
    pub verbose: bool,

    // Print version
    pub version: bool,
}

impl Arguments {
    /// Parses command-line arguments and builds an `Arguments` struct.
    pub fn build() -> WallSwitchResult<Arguments> {
        let args = Arguments::parse(std::env::args())?;

        if args.config {
            let config_path = get_config_path()?;
            let config = read_config_file(&config_path)?;
            let json: String = serde_json::to_string_pretty(&config)?;
            println!("{json}");
            std::process::exit(0);
        }

        Ok(args)
    }

    /// Parses command-line arguments into an `Arguments` struct.
    ///
    /// <https://stackoverflow.com/questions/51119143/how-do-i-check-the-second-element-of-the-command-line-arguments>
    fn parse(args: impl Iterator<Item = String>) -> WallSwitchResult<Self> {
        let mut arguments = Arguments::default();

        let args: Vec<String> = get_formatted_args(args);
        // println!("args: {args:?}");

        if args.is_empty() {
            return Ok(arguments);
        }

        let mut iter = args.into_iter();

        while let Some(current) = iter.next() {
            match current.as_ref() {
                "--min_size" | "-b" => {
                    let min_size: u64 = parse_value(iter.next(), "--min_size", 0)?;
                    arguments.min_size = Some(min_size)
                }
                "--max_size" | "-B" => {
                    let max_size: u64 = parse_value(iter.next(), "--max_size", 0)?;
                    arguments.max_size = Some(max_size)
                }
                "--min_dimension" | "-d" => {
                    let dimension: u64 = parse_value(iter.next(), "--min_dimension", 10)?;
                    arguments.min_dimension = Some(dimension)
                }
                "--max_dimension" | "-D" => {
                    let dimension: u64 = parse_value(iter.next(), "--max_dimension", 0)?;
                    arguments.max_dimension = Some(dimension)
                }
                "--interval" | "-i" => {
                    let interval: u64 = parse_value(iter.next(), "--interval", 5)?;
                    arguments.interval = Some(interval)
                }
                "--monitor" | "-m" => {
                    let value: u64 = parse_value(iter.next(), "--monitor", 1)?;
                    let monitor: u8 = value.try_into().map_err(WallSwitchError::from)?;
                    arguments.monitor = Some(monitor)
                }
                "--orientation" | "-o" => {
                    let orientation = parse_orientation(iter.next(), "--orientation")?;
                    arguments.monitor_orientation = Some(orientation)
                }
                "--pictures_per_monitor" | "-p" => {
                    let value: u64 = parse_value(iter.next(), "--pictures_per_monitor", 1)?;
                    let pictures_per_monitor: u8 =
                        value.try_into().map_err(WallSwitchError::from)?;
                    arguments.pictures_per_monitor = Some(pictures_per_monitor)
                }
                "--config" | "-c" => arguments.config = true,
                "--help" | "-h" => show_help_summary(),
                "--sort" | "-s" => arguments.sort = true,
                "--verbose" | "-v" => arguments.verbose = true,
                "--Version" | "-V" => show_version(),
                _ => return Err(UnexpectedArg { arg: current }),
            }
        }

        Ok(arguments)
    }
}

/// Formats command-line arguments to separate flags and values
fn get_formatted_args(args: impl Iterator<Item = String>) -> Vec<String> {
    args.skip(1) // skip program name
        // Split "--arg===12345" to "--arg 12345"
        .flat_map(|arg: String| {
            arg.split('=')
                .map(ToString::to_string)
                .collect::<Vec<String>>()
        })
        // Splits inclusive on the first digit: "--arg12345" to "--arg 12345"
        .flat_map(|arg: String| {
            if let Some(index) = arg.find(|c: char| c.is_ascii_digit()) {
                vec![arg[..index].to_string(), arg[index..].to_string()]
            } else {
                vec![arg]
            }
        })
        .map(|arg: String| arg.trim().to_string())
        .filter(|arg| !arg.is_empty())
        .collect()
}

/// Parse `Option<String>` to u64
///
/// Minimum dimension should be at least 10
///
/// Minimum interval should be at least 5 seconds
///
/// Monitor number should be at least 1
fn parse_value(opt_value: Option<String>, name: &'static str, min: u64) -> WallSwitchResult<u64> {
    if let Some(value) = opt_value {
        // println!("value: {value}");
        match value.parse::<u64>() {
            // num value should be at least min: num >= min
            Ok(num) if num >= min => Ok(num),
            Ok(_) => Err(AtLeastValue {
                arg: name.to_string(),
                value,
                num: min,
            }),
            Err(_) => Err(InvalidValue {
                arg: name.to_string(),
                value,
            }),
        }
    } else {
        Err(MissingValue {
            arg: name.to_string(),
        })
    }
}

/// Parse `Option<String>` to Orientation
fn parse_orientation(
    opt_string: Option<String>,
    name: &'static str,
) -> WallSwitchResult<Orientation> {
    if let Some(string) = opt_string {
        Orientation::from_str(&string)
    } else {
        Err(MissingValue {
            arg: name.to_string(),
        })
    }
}

/// Display help information with descriptions
fn show_help_summary() {
    let pkg_name = ENVIRON.get_pkg_name();
    let pkg_descr = env!("CARGO_PKG_DESCRIPTION");
    println!("{pkg_descr}");
    println!("\nUsage: {pkg_name} [OPTIONS]\n");
    println!("Options:\n");
    println!(
        "-b, --min_size <MIN_SIZE>\n\tSet a minimum file size (in bytes) for searching image files"
    );
    println!(
        "-B, --max_size <MAX_SIZE>\n\tSet a maximum file size (in bytes) for searching image files"
    );
    println!("-c, --config\n\tRead the configuration file and exit the program");
    println!(
        "-d, --min_dimension <MIN_DIMENSION>\n\tSet the minimum dimension that the height and width must satisfy"
    );
    println!(
        "-D, --max_dimension <MAX_DIMENSION>\n\tSet the maximum dimension that the height and width must satisfy"
    );
    println!("-h, --help\n\tPrint help");
    println!(
        "-i, --interval <INTERVAL>\n\tSet the interval (in seconds) between each wallpaper displayed"
    );
    println!("-m, --monitor <MONITOR_NUMBER>\n\tSet the number of monitors [default: 2]");
    println!(
        "-o, --orientation <ORIENTATION>\n\tInform monitor orientation: Horizontal (side-by-side) or Vertical (stacked)."
    );
    println!(
        "-p, --pictures_per_monitor <PICTURE>\n\tSet number of pictures (or images) per monitor [default: 1]"
    );
    println!("-s, --sort\n\tSort the images found");
    println!("-v, --verbose\n\tShow intermediate runtime messages");
    println!("-V, --version\n\tPrint version");
    std::process::exit(0);
}

fn show_version() {
    let pkg_name = ENVIRON.get_pkg_name();
    let pkg_version = env!("CARGO_PKG_VERSION");
    println!("{pkg_name} {pkg_version}");
    std::process::exit(0);
}

#[cfg(test)]
mod test_args_v2 {
    use crate::{Arguments, Orientation, WallSwitchResult};

    // cargo test -- --help
    // cargo test -- --nocapture get_arguments
    // cargo test -- --show-output filter_unique

    #[test]
    /// `cargo test --features args_v2 -- --show-output get_arguments`    
    fn get_arguments() -> WallSwitchResult<()> {
        let entries = [
            "program_name",
            "-i",
            "60",
            " -d === ",
            "200",
            "--config",
            "--monitor=",
            "3",
            "-m==5",
            "-c",
            "-p",
            "3",
            "--orientation",
            "horiZontal",
        ];

        println!("entries: {entries:?}");

        //let args = std::env::args();
        let args: Vec<String> = entries.iter().map(ToString::to_string).collect();

        let arguments = Arguments::parse(args.into_iter())?;
        println!("arguments: {arguments:#?}");

        let json: String = serde_json::to_string_pretty(&arguments)?;
        println!("arguments: {json}");

        assert_eq!(
            arguments,
            Arguments {
                config: true,
                monitor_orientation: Some(Orientation::Horizontal),
                min_dimension: Some(200),
                max_dimension: None,
                min_size: None,
                max_size: None,
                help: false,
                interval: Some(60),
                monitor: Some(5),
                pictures_per_monitor: Some(3),
                sort: false,
                verbose: false,
                version: false,
            }
        );

        Ok(())
    }

    #[test]
    /// `cargo test --features args_v2 -- --show-output split_arg_equal`
    fn split_arg_equal() -> WallSwitchResult<()> {
        let arg = "Löwe 老虎 Léo=öpard Gepa12345虎==rdi".to_string();
        println!("arg: {arg}");

        let result1: Vec<String> = if let Some(index) = arg.find('=') {
            vec![arg[..index].to_string(), arg[index + 1..].to_string()]
        } else {
            vec![arg.clone()]
        };

        let result2: Vec<&str> = arg
            .split_once('=')
            .into_iter()
            .flat_map(|(a, b)| vec![a, b])
            .collect();

        println!("result1: {result1:?}");

        assert_eq!(result1, ["Löwe 老虎 Léo", "öpard Gepa12345虎==rdi"]);
        assert_eq!(result1, result2);

        Ok(())
    }

    #[test]
    /// `cargo test --features args_v2 -- --show-output split_arg_digit`
    fn split_arg_digit() -> WallSwitchResult<()> {
        let arg = "Löwe 老虎 Léo=pard Gepa12345虎rdi".to_string();
        println!("arg: {arg}");

        let result = if let Some(index) = arg.find(|c: char| c.is_ascii_digit()) {
            //if let Some(index) = arg.chars().position(|c| c.is_ascii_digit()) {
            vec![arg[..index].to_string(), arg[index..].to_string()]
        } else {
            vec![arg]
        };

        println!("result: {result:?}");

        assert_eq!(result, ["Löwe 老虎 Léo=pard Gepa", "12345虎rdi"]);

        Ok(())
    }
}
