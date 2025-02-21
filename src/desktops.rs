use crate::{
    Config, FileInfo, MyResult,
    Orientation::{Horizontal, Vertical},
    U8Extension,
};
use std::{
    // cmp::Ordering,
    path::PathBuf,
    process::{Command, Output},
};

/// Set desktop wallpaper
pub fn set_wallpaper(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let desktop = &config.desktop;
    // println!("desktop: {desktop}");

    if desktop.contains("gnome") {
        set_gnome_wallpaper(images, config)?;
    } else if desktop.contains("xfce") {
        set_xfce_wallpaper(images, config)?;
    } else {
        set_openbox_wallpaper(images, config)?;
    }

    println!();

    Ok(())
}

fn set_xfce_wallpaper(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let monitors = get_xfce_monitors(config)?;

    if config.verbose {
        println!("monitors:\n{monitors:#?}");
    }

    for (image, monitor) in images.iter().zip(monitors) {
        apply_xfconf(&image.path, &monitor, config)?;
    }

    Ok(())
}

/**
    Get xfce monitors

    Example:
    ```
    // xfconf-query -c xfce4-desktop -p /backdrop -l | grep last-image
    // xfconf-query -c xfce4-desktop -p /backdrop -l | grep 'workspace0/last-image'

    let monitors = [
        "/backdrop/screen0/monitorDP-0/workspace0/last-image",
        "/backdrop/screen0/monitorDP-2/workspace0/last-image",
    ];
    ```
*/
fn get_xfce_monitors(config: &Config) -> MyResult<Vec<String>> {
    // Filter standard output that contains all these words
    let words = ["screen0", "workspace0", "last-image"];

    let mut cmd = Command::new("xfconf-query");
    let xfconf_cmd = cmd.args([
        "--channel",
        "xfce4-desktop",
        "--property",
        "/backdrop",
        "--list",
    ]);

    let xfconf_out: Output = exec_cmd(xfconf_cmd, config.verbose, "get_xfce_monitors: xfconf")?;

    let std_output: String = String::from_utf8(xfconf_out.stdout)?;

    let outputs: Vec<String> = std_output
        .trim()
        .split(['\n', ' '])
        .filter(|&output| words.into_iter().all(|word| output.contains(word)))
        .map(ToString::to_string)
        .collect();

    Ok(outputs)
}

fn apply_xfconf(path: &PathBuf, monitor: &str, config: &Config) -> MyResult<()> {
    let mut cmd = Command::new("xfconf-query");
    let xfconf = cmd
        .args(["--channel", "xfce4-desktop", "--property", monitor, "--set"])
        .arg(path);

    let msg = format!("apply_xfconf: xfconf {monitor}");

    exec_cmd(xfconf, config.verbose, &msg)?;

    Ok(())
}

fn set_openbox_wallpaper(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let mut feh_cmd = Command::new(&config.path_feh);

    for image in images {
        feh_cmd.arg("--bg-fill").arg(&image.path);
    }

    exec_cmd(&mut feh_cmd, config.verbose, "feh")?;

    Ok(())
}

/// Create a wallpaper file and set it as your desktop background image.
fn set_gnome_wallpaper(images: &[FileInfo], config: &Config) -> MyResult<()> {
    // Create a wallpaper file with magick command line (ImageMagick).
    create_background_image(images, config)?;

    // gsettings set org.gnome.desktop.background picture-uri      '/home/use_name/wallswitch.jpg'
    // gsettings set org.gnome.desktop.background picture-uri-dark '/home/use_name/wallswitch.jpg'

    for picture in ["picture-uri", "picture-uri-dark"] {
        let mut cmd = Command::new("gsettings");
        let gsettings = cmd
            .args(["set", "org.gnome.desktop.background", picture])
            .arg(&config.wallpaper);

        let msg = format!("gsettings {picture}");

        exec_cmd(gsettings, config.verbose, &msg)?;
    }

    // gsettings set org.gnome.desktop.background picture-options spanned

    let mut cmd = Command::new("gsettings");
    let spanned = cmd.args([
        "set",
        "org.gnome.desktop.background",
        "picture-options",
        "spanned",
    ]);

    exec_cmd(spanned, config.verbose, "spanned")?;

    Ok(())
}

/**
Create custom background image

To join images horizontally: +append
To join images vertically: -append
To see gravity options: magick -list gravity

### Example.
Consider 3 images in the directory: "fig01.webp", "fig02.avif" and "fig03.jpg".

Two distinct cases:

- case 1. N Monitors with the same resolution (3840x2160):

magick fig0* -gravity Center -resize 3840x2160^ -extent 3840x2160 +append wallpaper.jpg

or with aspect ratio: 16:9

magick fig0* -gravity Center -resize 3840x2160^ -extent 16:9 +append wallpaper.jpg

- case 2. 3 Monitors with different resolutions (3840x2160, 1920x1080 and 3840x2160):

magick fig01* -gravity Center -resize 3840x2160^ -extent 3840x2160 wallpaper_01.jpg \
magick fig02* -gravity Center -resize 1920x1080^ -extent 1920x1080 wallpaper_02.jpg \
magick fig03* -gravity Center -resize 3840x2160^ -extent 3840x2160 wallpaper_03.jpg \
magick -gravity South wallpaper_0*.jpg +append wallpaper.jpg

ImageMagick can run multiple operations on separate instances in a single command:

magick -gravity Center \
\( fig01* -resize 3840x2160^ -extent 3840x2160 \) \
\( fig02* -resize 1920x1080^ -extent 1920x1080 \) \
\( fig03* -resize 3840x2160^ -extent 3840x2160 \) \
-gravity South +append wallpaper.jpg

<https://www.imagemagick.org/script/command-line-processing.php>
*/
fn create_background_image(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let mut magick_cmd = Command::new(&config.path_magick);

    get_partitions_iter(images, config)
        .zip(&config.monitors)
        .try_for_each(|(images, monitor)| -> MyResult<()> {
            let mut width: u64 = monitor.resolution.width;
            let mut height: u64 = monitor.resolution.height;

            let pictures_per_monitor = monitor.pictures_per_monitor.to_u64();

            let remainder_w: usize = (width % pictures_per_monitor).try_into()?;
            let remainder_h: usize = (height % pictures_per_monitor).try_into()?;

            match monitor.picture_orientation {
                Horizontal => height /= pictures_per_monitor,
                Vertical => width /= pictures_per_monitor,
            }

            magick_cmd.args(["(", "-gravity", "Center"]);

            images.iter().enumerate().for_each(|(index, image)| {
                let mut w = width;
                let mut h = height;

                // Add extra row or column to adjust image composition to resolution.
                match monitor.picture_orientation {
                    Horizontal => {
                        if index < remainder_h {
                            h += 1; // Add extra row if necessary
                        }
                    }
                    Vertical => {
                        if index < remainder_w {
                            w += 1; // Add extra column if necessary
                        }
                    }
                }

                let resize = format!("{w}x{h}^");
                let extent = format!("{w}x{h}");

                magick_cmd
                    .arg("(")
                    .arg(&image.path)
                    .args(["-resize", &resize])
                    .args(["-extent", &extent])
                    .arg(")");
            });

            // Indicates how the images are combined
            match monitor.picture_orientation {
                Horizontal => {
                    magick_cmd.args(["-gravity", "South", "-append", ")"]);
                }
                Vertical => {
                    magick_cmd.args(["-gravity", "South", "+append", ")"]);
                }
            }

            Ok(())
        })?;

    match config.monitor_orientation {
        Horizontal => {
            magick_cmd.arg("+append").arg(&config.wallpaper);
        }
        Vertical => {
            magick_cmd.arg("-append").arg(&config.wallpaper);
        }
    }

    exec_cmd(&mut magick_cmd, config.verbose, "magick")?;

    Ok(())
}

/// Get partitions from a Slice
#[allow(dead_code)]
fn get_partitions_slice<'a>(mut images: &'a [FileInfo], config: &'a Config) -> Vec<&'a [FileInfo]> {
    let mut partition = Vec::new();

    config.monitors.iter().for_each(|monitor| {
        let (head, tail) = images.split_at(monitor.pictures_per_monitor.into());
        images = tail;
        partition.push(head);
    });

    partition
}

/// Returns an iterator over partitions of images based on monitor settings.
///
/// Arguments:
///
/// * `images`: A reference to a slice of `FileInfo` objects, representing the images to be partitioned.
/// * `config`: A reference to a `Config` object, containing information about the monitors and their settings.
fn get_partitions_iter<'a>(
    mut images: &'a [FileInfo],
    config: &'a Config,
) -> impl Iterator<Item = &'a [FileInfo]> {
    // Create an iterator over the monitor configurations
    config.monitors.iter().map(move |monitor| {
        let (head, tail) = images.split_at(monitor.pictures_per_monitor.into());
        images = tail;
        head
    })
}

/// Executes the command as a child process,
/// waiting for it to finish and collecting all of its output.
pub fn exec_cmd(cmd: &mut Command, verbose: bool, msg: &str) -> MyResult<Output> {
    let output: Output = cmd.output().inspect_err(|error| {
        eprintln!("fn exec_cmd()");
        eprintln!("cmd: {cmd:?}");
        eprintln!("Error: {error}");
    })?;

    if !output.status.success() || verbose {
        let program = cmd.get_program();
        let arguments: Vec<_> = cmd.get_args().collect();

        println!("\nprogram: {program:?}");
        println!("arguments: {arguments:#?}");

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !stdout.trim().is_empty() {
            println!("stdout:'{}'\n", stdout.trim());
        }
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let status = output.status;

        eprintln!("{msg} status: {status}");
        eprintln!("{msg} stderr: {stderr}");

        panic!("{:?}", stderr);
    }

    Ok(output)
}
