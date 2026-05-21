use crate::{
    Colors, Orientation, ProceduralEffect, SortCriteria, WallSwitchResult, get_config_path,
    read_config_file,
};
use clap::{
    CommandFactory, Parser,
    builder::styling::{AnsiColor, Effects, Styles},
}; // command-line arguments
use clap_complete::{Generator, Shell, generate};

/// Custom Clap styling to mimic a beautiful colored help menu.
fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Cyan.on_default())
}

/// Dynamically builds the extra help menu section with named colors
/// and injects the actual system path of the config file.
fn get_after_help() -> String {
    // Safely attempt to get the configuration path, defaulting to a generic string on failure
    let config_path = get_config_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "~/.config/wallswitch/wallswitch.json".to_string());

    // 1. Initialize the base string with the configuration file path
    let mut help_text = format!(
        "{}\n  {}\n\n{}\n",
        "Config file:".yellow().bold(),
        config_path.blue().bold(),
        "Examples:".yellow().bold()
    );

    // 2. Define examples as an Array of structured Tuples ("Comment", "Command")
    let examples = [
        (
            "# Start the automatic background loop using default settings",
            "wallswitch",
        ),
        (
            "# Run a single wallpaper update cycle and exit (useful for cron jobs)",
            "wallswitch --once",
        ),
        (
            "# Change wallpaper every 10 minutes (600 seconds)",
            "wallswitch --interval 600",
        ),
        (
            "# Set 3 different wallpapers per monitor (Gnome desktop only)",
            "wallswitch --pictures_per_monitor 3",
        ),
        (
            "# Filter images by dimension (min 1080px) and file size (max 5MB)",
            "wallswitch --min_dimension 1080 --max_size 5242880",
        ),
        (
            "# Apply randomized procedural overlays (Julia fractal or starfield) onto the selected wallpapers",
            "wallswitch --effect random",
        ),
        (
            "# Apply a specific procedural Julia fractal overlay",
            "wallswitch --effect fractal",
        ),
        (
            "# Dry run mode to see what would be executed without applying changes",
            "wallswitch --dry-run --verbose",
        ),
        (
            "# Wayland (awww): Use specific transition effects and duration",
            "wallswitch --transition-type wave --transition-duration 3",
        ),
        (
            "# List all found images sorted by file size",
            "wallswitch --list size",
        ),
        (
            "# Display all processed images (with dimensions) in JSON format",
            "wallswitch --list processed",
        ),
        (
            "# Display all images that haven't been probed yet",
            "wallswitch --list unprocessed",
        ),
        (
            "# Count processed images using jq",
            "wallswitch -l processed | jq 'length'",
        ),
    ];

    // 3. Iterate over the list, applying colors centrally and idiomatically
    for (comment, cmd) in examples {
        help_text.push_str(&format!(
            "  {}\n  {}\n\n",
            comment.dimmed(),
            cmd.green().bold()
        ));
    }

    // Remove trailing newlines at the end of the string
    help_text.trim_end().to_string()
}

const APPLET_TEMPLATE: &str = "\
{before-help}
{about}
{usage-heading} {usage}

{all-args}
{after-help}";

/// Command line arguments
#[derive(Parser, Debug, Clone)]
#[command(
    // Read from `Cargo.toml`
    author, version, about,
    long_about = None,
    next_line_help = true,
    help_template = APPLET_TEMPLATE,
    styles = get_styles(),
    after_help = get_after_help(),
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

    /// Apply a procedural overlay effect to the selected wallpapers before displaying.
    #[arg(
        short('e'),
        long("effect"),
        value_enum,
        required = false,
        default_value = None,
        hide_default_value = true,
    )]
    pub effect: Option<ProceduralEffect>,

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

    /// List all found images and exit.
    ///
    /// Sort criteria: [path, size, sizedesc, name, extension, width, height, area, ratio, time, processed, unprocessed, cache]
    #[arg(short('l'), long("list"), value_name = "CRITERIA")]
    pub list: Option<SortCriteria>,

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

    /// Run a single wallpaper update cycle and exit.
    #[arg(long, default_value_t = false)]
    pub once: bool,

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
        value_parser = clap::value_parser!(u8).range(1..=256)
    )]
    pub pictures_per_monitor: Option<u8>,

    /// Sort the images found.
    #[arg(short('s'), long("sort"), default_value_t = false)]
    pub sort: bool,

    /// Run without applying the wallpapers (simulation mode).
    #[arg(long("dry-run"), default_value_t = false)]
    pub dry_run: bool,

    /// Transition type for Wayland compositors using awww (e.g. wipe, wave, fade, random).
    #[arg(long("transition-type"), required = false)]
    pub transition_type: Option<String>,

    /// Duration of the transition animation in seconds.
    #[arg(long("transition-duration"), required = false)]
    pub transition_duration: Option<u16>,

    /// Frames per second for transition smoothness.
    #[arg(long("transition-fps"), required = false)]
    pub transition_fps: Option<u16>,

    /// Angle used by directional transitions (wipe, wave).
    #[arg(long("transition-angle"), required = false)]
    pub transition_angle: Option<u16>,

    /// Origin position used by grow/outer transitions (e.g. center, top).
    #[arg(long("transition-pos"), required = false)]
    pub transition_pos: Option<String>,

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
