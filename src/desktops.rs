use std::{
    // cmp::Ordering,
    path::PathBuf,
    process::{Command, Output},
};

use crate::{Config, FileInfo, FileInfoExt, FloatIterExt, MyResult, WSError::MinValue};

/// Set wallpaper side by side
pub fn set_wallpaper_side_by_side(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let desktop = &config.desktop;
    //println!("desktop: {desktop}");

    if desktop.contains("gnome") || desktop.contains("ubuntu") {
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
    let xfconf_cmd = cmd
        .arg("--channel")
        .arg("xfce4-desktop")
        .arg("--property")
        .arg("/backdrop")
        .arg("--list");

    let xfconf_out: Output = exec_cmd(xfconf_cmd, config, "get_xfce_monitors: xfconf")?;

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
        .arg("--channel")
        .arg("xfce4-desktop")
        .arg("--property")
        .arg(monitor)
        .arg("--set")
        .arg(path);

    let msg = format!("apply_xfconf: xfconf {monitor}");

    exec_cmd(xfconf, config, &msg)?;

    Ok(())
}

fn set_openbox_wallpaper(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let mut feh_cmd = Command::new("feh");

    for image in images {
        feh_cmd.arg("--bg-fill").arg(&image.path);
    }

    exec_cmd(&mut feh_cmd, config, "feh")?;

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
            .arg("set")
            .arg("org.gnome.desktop.background")
            .arg(picture)
            .arg(&config.wallpaper);

        let msg = format!("gsettings {picture}");

        exec_cmd(gsettings, config, &msg)?;
    }

    // gsettings set org.gnome.desktop.background picture-options spanned

    let mut cmd = Command::new("gsettings");
    let spanned = cmd
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-options")
        .arg("spanned");

    exec_cmd(spanned, config, "spanned")?;

    Ok(())
}

/// Create custom background image
fn create_background_image(images: &[FileInfo], config: &Config) -> MyResult<()> {
    let width_min: u64 = images.get_width_min().ok_or(MinValue)?;
    let resizes: Vec<f64> = get_horizontal_resizing(images, width_min);
    let paths: Vec<PathBuf> = get_formatted_paths(images, width_min, &resizes);

    let width: u64 = width_min * (images.len() as u64);
    let height: f64 = resizes.into_iter().float_min();
    let crop = format!("{width}x{height}+0+0");

    // Example with 2 monitors:
    // magick path0 path1 -gravity Center -crop 7108x1999+0+0 +append wallpaper_file_path

    let mut magick_cmd = Command::new("magick");

    for path in paths {
        magick_cmd.arg(path);
    }

    magick_cmd
        .arg("-gravity")
        .arg("Center")
        .arg("-crop")
        .arg(crop)
        .arg("+append")
        .arg(&config.wallpaper);

    exec_cmd(&mut magick_cmd, config, "magick")?;

    Ok(())
}

fn get_horizontal_resizing(images: &[FileInfo], width_min: u64) -> Vec<f64> {
    let mut resizes: Vec<f64> = images
        .iter()
        .map(|image| (width_min * image.dimension.height) as f64 / image.dimension.width as f64)
        .collect();

    // Run the xrandr command:
    // xrandr --listactivemonitors
    // xrandr | grep 'connected primary'
    // connected primary 3840x2160+0+0
    let aspect_ratio = (width_min as f64) * (2160.0 / 3840.0);

    resizes.push(aspect_ratio);

    resizes
}

/// Get formatted paths
///
/// magick needs formatted paths
fn get_formatted_paths(images: &[FileInfo], width_min: u64, resizes: &[f64]) -> Vec<PathBuf> {
    images
        .iter()
        .zip(resizes.iter())
        .map(|(image, resize)| {
            if image.dimension.width > width_min {
                format!("{}[{width_min}x{resize}]", image.path.display()).into()
            } else {
                image.path.clone()
            }
        })
        .collect()
}

/*
// Example with 2 monitors
fn get_formatted_paths_v2(images: &[FileInfo], width_min: u64, resizes: &[f64]) -> Vec<PathBuf> {
    let widths: Vec<&u64> = images.iter().map(|image| &image.dimension.width).collect();
    let mut paths: Vec<PathBuf> = images.iter().map(|image| image.path.clone()).collect();

    match widths[0].cmp(widths[1]) {
        // Example: magick path0[4879x2744.0668] path1 -gravity Center -crop 9758x2744.0668+0+0 +append file_path
        Ordering::Greater => {
            let path0 = format!("{}[{}x{}]", paths[0].display(), width_min, resizes[0]);
            paths[0] = path0.into();
            paths
        }
        // Example: magick path0 path1[3554x2452.9690] -gravity Center -crop 7108x1999+0+0 +append file_path
        Ordering::Less => {
            let path1 = format!("{}[{}x{}]", paths[1].display(), width_min, resizes[1]);
            paths[1] = path1.into();
            paths
        }
        // Example: magick path0 path1 -gravity Center -crop 7108x1999+0+0 +append file_path
        Ordering::Equal => paths,
    }
}
*/

/// Executes the command as a child process,
/// waiting for it to finish and collecting all of its output.
pub fn exec_cmd(cmd: &mut Command, config: &Config, msg: &str) -> MyResult<Output> {
    let output: Output = cmd.output()?;

    if !output.status.success() || config.verbose {
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
