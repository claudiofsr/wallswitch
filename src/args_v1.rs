// use clap
// cargo b -r && cargo install --path=. --features args_v1

use crate::{Orientation, WallSwitchResult, get_config_path, read_config_file};
use anstyle::{
    AnsiColor::{Cyan, Green, Yellow},
    Color::Ansi,
    Style,
};
use clap::{CommandFactory, Parser}; // command-line arguments
use clap_complete::{Generator, Shell, generate};

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

    /// Read the configuration file and exit the program.
    #[arg(short('c'), long("config"), default_value_t = false)]
    pub config: bool,

    /**
    Generate shell completions and exit the program.

    ### How to generate shell completions for Z-shell:

    #### Example 1 (as a regular user):
    Generate completion_derive.zsh file with:

    ```console

        wallswitch --generate=zsh > completion_derive.zsh

    ```

    Append the contents of the completion_derive.zsh file to the end of completion zsh file.

    ZSH completions are commonly stored in any directory listed in your `$fpath` variable.

    On Linux, view `$fpath` variable with:

    ```console

        echo $fpath | perl -nE 'say for split /\s+/'

    ```

    And then, execute:

    ```console

        compinit && zsh

    ```

    #### Example 2 (as a regular user):
    Generate completions to rustup and wallswitch.

    Visible to only the regular user.

    ```console

        mkdir -p ~/.oh-my-zsh/functions

        rustup completions zsh > ~/.oh-my-zsh/functions/_rustup

        wallswitch --generate=zsh > ~/.oh-my-zsh/functions/_wallswitch

        compinit && zsh

    ```

    #### Example 3 (as root):

    Generate completions to rustup, cargo and wallswitch.

    Visible to all system users.

    ```console

        mkdir -p /usr/local/share/zsh/site-functions

        rustup completions zsh > /usr/local/share/zsh/site-functions/_rustup

        rustup completions zsh cargo > /usr/local/share/zsh/site-functions/_cargo

        wallswitch --generate=zsh > /usr/local/share/zsh/site-functions/_wallswitch

        compinit && zsh

    ```

    See `rustup completions` for detailed help.

    <https://github.com/clap-rs/clap/blob/master/clap_complete/examples/completion-derive.rs>
    */
    #[arg(short('g'), long("generate"), value_enum)]
    pub generator: Option<Shell>,

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
        short('m'), long("monitor"),
        required = false,
        default_value = None,
        hide_default_value = true,
        value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub monitor: Option<u8>,

    /// Inform monitor orientation: Horizontal (side-by-side) or Vertical (stacked).
    ///
    /// Orientation: [Horizontal, Vertical]
    ///
    /// Default orientation: Horizontal.
    #[arg(
        short('o'),
        long("orientation"),
        required = false,
        default_value = None,
        hide_default_value = true,
    )]
    pub monitor_orientation: Option<Orientation>,

    /// Set number of pictures (or images) per monitor [default: 1]
    ///
    /// Each monitor can have a diferent number of pictures (or images)
    ///
    /// Gnome desktop only
    #[arg(
        short('p'), long("pictures_per_monitor"),
        required = false,
        default_value = None,
        hide_default_value = true,
        value_parser = clap::value_parser!(u8).range(1..)
    )]
    pub pictures_per_monitor: Option<u8>,

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
    pub fn build() -> WallSwitchResult<Arguments> {
        let args: Arguments = Arguments::parse();

        if let Some(generator) = args.generator {
            args.print_completions(generator);
        }

        if args.config {
            let config_path = get_config_path()?;
            let config = read_config_file(&config_path)?;
            let json: String = serde_json::to_string_pretty(&config)?;
            println!("{json}");
            std::process::exit(0);
        }

        Ok(args)
    }

    /// Print shell completions to standard output
    fn print_completions<G>(&self, r#gen: G)
    where
        G: Generator + std::fmt::Debug,
    {
        let mut cmd = Arguments::command();
        let cmd_name = cmd.get_name().to_string();
        let mut stdout = std::io::stdout();

        eprintln!("Generating completion file for {gen:?}...");
        generate(r#gen, &mut cmd, cmd_name, &mut stdout);
        std::process::exit(1);
    }
}
