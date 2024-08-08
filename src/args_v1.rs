// use clap
// cargo b -r && cargo install --path=. --features args_v1

use crate::{get_config_path, read_config_file, MyResult};
use anstyle::{
    AnsiColor::{Cyan, Green, Yellow},
    Color::Ansi,
    Style,
};
use clap::Parser; // command-line arguments

// https://stackoverflow.com/questions/74068168/clap-rs-not-printing-colors-during-help
fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .placeholder(Style::new().fg_color(Some(Ansi(Yellow))))
        .usage(Style::new().fg_color(Some(Ansi(Cyan))).bold())
        .header(Style::new().fg_color(Some(Ansi(Cyan))).bold().underline())
        .literal(Style::new().fg_color(Some(Ansi(Green))))
}

/// Command line arguments
#[derive(Parser, Debug, Clone)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    styles=get_styles(),
)]
pub struct Arguments {
    /// Read the configuration file and exit the program.
    #[arg(short('c'), long("config"), default_value_t = false)]
    pub config: bool,

    /// Set the minimum dimension that the height and width must satisfy.
    ///
    /// width >= min_dimension && height >= min_dimension
    ///
    /// Default value: min_dimension = 600.
    #[arg(
        short('d'), long("min_dimension"),
        required = false,
        default_value = None,
        hide_default_value = true,
        value_parser = clap::value_parser!(u64).range(10..)
    )]
    pub min_dimension: Option<u64>,

    /// Set the maximum dimension that the height and width must satisfy.
    ///
    /// width <= max_dimension && height <= max_dimension
    #[arg(
        short('D'), long("max_dimension"),
        required = false,
        default_value = None,
        hide_default_value = true
    )]
    pub max_dimension: Option<u64>,

    // see config::config_boundary() values
    /// Set a minimum file size (in bytes) for searching image files.
    ///
    /// keep files whose size is greater than or equal to a minimum value.
    ///
    /// size >= min_size
    #[arg(
        short('b'), long("min_size"),
        required = false,
        default_value = None,
        hide_default_value = true,
    )]
    pub min_size: Option<u64>,

    /// Set a maximum file size (in bytes) for searching image files.
    ///
    /// keep files whose size is less than or equal to a maximum value.
    ///
    /// size <= max_size
    #[arg(
        short('B'), long("max_size"),
        required = false,
        default_value = None,
        hide_default_value = true
    )]
    pub max_size: Option<u64>,

    /// Set the interval (in seconds) between each wallpaper displayed.
    ///
    /// Default value: interval = 30 * 60 = 1800 seconds (30 minutes).
    #[arg(
        short('i'), long("interval"),
        required = false,
        default_value = None,
        hide_default_value = true,
        value_parser = clap::value_parser!(u64).range(5..)
    )]
    pub interval: Option<u64>,

    /// Set the number of monitors [default: 2]
    #[arg(
        short('n'), long("monitor"),
        required = false,
        default_value = None,
        hide_default_value = true,
        value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub monitor: Option<u8>,

    /// Sort the images found.
    #[arg(short('s'), long("sort"), default_value_t = false)]
    pub sort: bool,

    /// Show intermediate runtime messages.
    ///
    /// Show found images.
    ///
    /// Show pid numbers of previous running program.
    #[arg(short('v'), long("verbose"), default_value_t = false)]
    pub verbose: bool,
}

impl Arguments {
    /// Build Arguments struct
    pub fn build() -> MyResult<Arguments> {
        let args: Arguments = Arguments::parse();

        if args.config {
            let config_path = get_config_path()?;
            let config = read_config_file(&config_path)?;
            let json: String = serde_json::to_string_pretty(&config)?;
            println!("{json}");
            std::process::exit(0);
        }

        Ok(args)
    }
}
