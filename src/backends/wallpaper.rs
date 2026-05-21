use crate::{
    AwwwBackend, Config, Desktop, FileInfo, FractalGenerator,
    Orientation::{Horizontal, Vertical},
    ProceduralEffect, StarfieldGenerator, U8Extension, WallSwitchError, WallSwitchResult,
    detect_monitors, get_random_integer, is_installed,
};
use image::{RgbImage, imageops::FilterType};
use std::process::{Command, Output};

/// Core trait defining the wallpaper application logic.
/// Follows the "Functional Core, Imperative Shell" pattern.
pub trait WallpaperBackend {
    /// PURE FUNCTION: Only constructs the required system commands.
    /// Does NOT execute them. This makes the logic highly testable and predictable.
    fn build_commands(images: &[FileInfo], config: &Config) -> WallSwitchResult<Vec<Command>>;

    /// IMPURE FUNCTION: Executes the built commands.
    /// It defaults to sequentially running `build_commands`, but can be
    /// overridden by compositors that require complex state checks
    /// (e.g., Hyprland preloading, Swaybg daemon spawning).
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let mut commands = Self::build_commands(images, config)?;
        for cmd in commands.iter_mut() {
            let program_name = cmd.get_program().to_string_lossy().to_string();
            exec_cmd(cmd, config.verbose, &format!("Executing {program_name}"))?;
        }
        Ok(())
    }
}

/// Set desktop wallpaper based on the detected Desktop Environment.
pub fn set_wallpaper(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
    // 1. Determine if compilation is needed (if effect is active, or Gnome, or P > 1)
    let needs_compilation = config.desktop == Desktop::Gnome
        || config.effect != ProceduralEffect::None
        || config.monitors.iter().any(|m| m.pictures_per_monitor > 1);

    let compiled_images = if needs_compilation {
        compile_wallpapers_for_monitors(images, config)?
    } else {
        images.to_vec()
    };

    // 2. Dispatch to the appropriate backend using the compiled single-image-per-monitor files
    match config.desktop {
        Desktop::Gnome => {
            // Gnome requires stitching the compiled monitor images together into a single spanned file
            let mut monitor_canvases = Vec::new();
            for img_info in &compiled_images {
                let img = image::open(&img_info.path)
                    .map_err(|e| {
                        WallSwitchError::UnableToFind(format!(
                            "Failed to load compiled monitor canvas: {e}"
                        ))
                    })?
                    .to_rgb8();
                monitor_canvases.push(img);
            }
            let final_wallpaper = assemble_final_wallpaper(&monitor_canvases, config)?;
            final_wallpaper
                .save(&config.wallpaper)
                .map_err(|e| WallSwitchError::Io(std::io::Error::other(e)))?;

            if config.verbose {
                println!("Stitched wallpaper saved to Gnome: {:?}", config.wallpaper);
            }

            GnomeBackend::apply(&compiled_images, config)?;
        }
        Desktop::Xfce => XfceBackend::apply(&compiled_images, config)?,
        Desktop::Hyprland => HyprlandBackend::apply(&compiled_images, config)?,

        Desktop::Niri | Desktop::Labwc | Desktop::Mango | Desktop::Wayland => {
            if is_installed("awww") {
                AwwwBackend::apply(&compiled_images, config)?;
            } else if is_installed("swaybg") {
                SwaybgBackend::apply(&compiled_images, config)?;
            } else if is_installed("hyprpaper") {
                HyprlandBackend::apply(&compiled_images, config)?;
            } else {
                return Err(WallSwitchError::MissingWaylandTools);
            }
        }

        Desktop::Openbox => OpenboxBackend::apply(&compiled_images, config)?,
    }

    Ok(())
}

// ==============================================================================
// BACKEND IMPLEMENTATIONS
// ==============================================================================

pub struct GnomeBackend;

impl WallpaperBackend for GnomeBackend {
    fn build_commands(_images: &[FileInfo], config: &Config) -> WallSwitchResult<Vec<Command>> {
        let mut commands = Vec::new();

        // GSettings commands to set the background picture URIs
        for picture in ["picture-uri", "picture-uri-dark"] {
            let mut cmd = Command::new("gsettings");
            cmd.args(["set", "org.gnome.desktop.background", picture])
                .arg(&config.wallpaper);
            commands.push(cmd);
        }

        // GSettings command to set the picture options to spanned
        let mut cmd = Command::new("gsettings");
        cmd.args([
            "set",
            "org.gnome.desktop.background",
            "picture-options",
            "spanned",
        ]);
        commands.push(cmd);

        Ok(commands)
    }
}

pub struct XfceBackend;

impl WallpaperBackend for XfceBackend {
    fn build_commands(images: &[FileInfo], config: &Config) -> WallSwitchResult<Vec<Command>> {
        let mut commands = Vec::new();
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}");
        }

        // Cycle through compiled single-image-per-monitor backgrounds
        for (image, monitor) in images.iter().cycle().zip(monitors) {
            let mut cmd = Command::new("xfconf-query");
            cmd.args([
                "--channel",
                "xfce4-desktop",
                "--property",
                &monitor,
                "--create",
                "--type",
                "string",
                "--set",
            ])
            .arg(&image.path);

            commands.push(cmd);
        }

        Ok(commands)
    }
}

pub struct OpenboxBackend;

impl WallpaperBackend for OpenboxBackend {
    fn build_commands(images: &[FileInfo], config: &Config) -> WallSwitchResult<Vec<Command>> {
        let mut feh_cmd = Command::new(&config.path_feh);

        for image in images {
            feh_cmd.arg("--bg-fill").arg(&image.path);
        }

        Ok(vec![feh_cmd])
    }
}

pub struct SwaybgBackend;

impl WallpaperBackend for SwaybgBackend {
    fn build_commands(_images: &[FileInfo], _config: &Config) -> WallSwitchResult<Vec<Command>> {
        Ok(vec![])
    }

    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}");
        }

        let _ = Command::new("pkill").arg("swaybg").output();

        let mut cmd = Command::new("swaybg");
        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image.path.to_str().unwrap_or_default();
            cmd.arg("-o")
                .arg(monitor)
                .arg("-i")
                .arg(path_str)
                .arg("-m")
                .arg("fill");
        }

        if config.verbose {
            let program = cmd.get_program();
            let arguments: Vec<_> = cmd.get_args().collect::<Vec<_>>();
            println!("\nprogram: {program:?}");
            println!("arguments: {arguments:#?}");
        }

        cmd.stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(WallSwitchError::Io)?;

        Ok(())
    }
}

pub struct HyprlandBackend;

impl WallpaperBackend for HyprlandBackend {
    fn build_commands(_images: &[FileInfo], _config: &Config) -> WallSwitchResult<Vec<Command>> {
        Ok(vec![])
    }

    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let monitors = detect_monitors(config)?;

        if config.verbose {
            println!("monitors:\n{monitors:#?}");
        }

        let mut check_cmd = Command::new("hyprctl");
        check_cmd.args(["hyprpaper", "listloaded"]);

        let loaded_str = match check_cmd.output() {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => {
                return Err(WallSwitchError::UnableToFind(
                    "hyprpaper daemon not running".into(),
                ));
            }
        };

        for (image, monitor) in images.iter().cycle().zip(&monitors) {
            let path_str = image.path.to_str().unwrap_or_default();

            if !loaded_str.contains(path_str) {
                let mut preload_cmd = Command::new("hyprctl");
                preload_cmd.args(["hyprpaper", "preload", path_str]);

                if config.verbose {
                    println!("\nprogram: {:?}", preload_cmd.get_program());
                    println!(
                        "arguments: {:#?}",
                        preload_cmd.get_args().collect::<Vec<_>>()
                    );
                }
                let _ = preload_cmd.output();
            }

            let mut wall_cmd = Command::new("hyprctl");
            let wall_arg = format!("{monitor},{path_str}");
            wall_cmd.args(["hyprpaper", "wallpaper", &wall_arg]);

            exec_cmd(
                &mut wall_cmd,
                config.verbose,
                &format!("Apply wallpaper on {monitor}"),
            )?;
        }

        let mut unload_cmd = Command::new("hyprctl");
        unload_cmd.args(["hyprpaper", "unload", "unused"]);
        let _ = unload_cmd.output();

        Ok(())
    }
}

// ==============================================================================
// STRUCTURAL & MATHEMATICAL GEOMETRY COMPUTATIONS (Pure Helpers)
// ==============================================================================

struct LayoutTarget {
    base_w: u64,
    base_h: u64,
    rem_w: usize,
    rem_h: usize,
}

impl LayoutTarget {
    fn calculate(monitor: &crate::Monitor) -> Result<Self, std::num::TryFromIntError> {
        let mut width = monitor.resolution.width;
        let mut height = monitor.resolution.height;
        let pics_per_monitor = monitor.pictures_per_monitor.to_u64();

        let rem_w = (width % pics_per_monitor).try_into()?;
        let rem_h = (height % pics_per_monitor).try_into()?;

        match monitor.picture_orientation {
            Horizontal => height /= pics_per_monitor,
            Vertical => width /= pics_per_monitor,
        }

        Ok(Self {
            base_w: width,
            base_h: height,
            rem_w,
            rem_h,
        })
    }
}

/// Helper function to select and apply procedural overlays in-memory.
fn apply_selected_effect(
    canvas: &mut RgbImage,
    monitor: &crate::Monitor,
    config: &Config,
    index: usize,
) {
    if config.effect == ProceduralEffect::None {
        return;
    }

    // Resolve the selected effect choice (handles random automatically)
    match config.effect.resolve() {
        ProceduralEffect::JuliaFractal => {
            let generator = FractalGenerator::random();
            if config.verbose {
                println!(
                    "Applying randomized Julia fractal (c = {} + {}i, zoom = {}) to Monitor {index}",
                    generator.c_re, generator.c_im, generator.zoom
                );
            }
            generator.apply_effect_in_memory(canvas);
        }
        ProceduralEffect::Starfield => {
            let star_count = get_random_integer(60, 120) as usize;
            let starfield = StarfieldGenerator::new(
                star_count,
                monitor.resolution.width as u32,
                monitor.resolution.height as u32,
            );
            if config.verbose {
                println!(
                    "Applying Starfield / Bokeh overlay ({star_count} stars) to Monitor {index}"
                );
            }
            starfield.apply_effect_in_memory(canvas);
        }
        _ => {} // ProceduralEffect::None and ProceduralEffect::Random are resolved before reaching here
    }
}

/// Compiles a single monitor canvas, applies overlays, saves the output to disk, and builds its FileInfo metadata.
fn compile_single_monitor_background(
    partition: &[FileInfo],
    monitor: &crate::Monitor,
    config: &Config,
    index: usize,
) -> WallSwitchResult<FileInfo> {
    // 1. Assemble separate pictures into a single composite monitor background in-memory
    let mut monitor_canvas = assemble_monitor_canvas(partition, monitor)?;

    // 2. Overlay dynamic procedural adjustments if any are requested
    if config.effect != ProceduralEffect::None {
        apply_selected_effect(&mut monitor_canvas, monitor, config, index);
    }

    // 3. Determine unique temporary file path for this monitor background
    let output_path = std::env::temp_dir().join(format!("wallswitch_monitor_{index}.jpg"));

    // 4. Save compiled monitor canvas to disk
    monitor_canvas
        .save(&output_path)
        .map_err(|e| WallSwitchError::Io(std::io::Error::other(e)))?;

    if config.verbose {
        println!("Monitor {index} background assembled: {:?}", output_path);
    }

    // 5. Construct structural metadata representing the updated target file
    Ok(FileInfo {
        path: output_path,
        size: 0,
        mtime: 0,
        hash: String::new(),
        dimension: Some(crate::Dimension {
            width: monitor.resolution.width,
            height: monitor.resolution.height,
        }),
        is_valid: Some(true),
        number: index + 1,
        total: config.monitors.len(),
    })
}

/// Pre-processes and compiles separate multi-picture composite backgrounds in parallel for each monitor.
pub fn compile_wallpapers_for_monitors(
    images: &[FileInfo],
    config: &Config,
) -> WallSwitchResult<Vec<FileInfo>> {
    if config.verbose {
        println!("Assembling multi-monitor wallpaper in pure Rust ...");
    }

    let partitions: Vec<_> = get_partitions_iter(images, config).collect();
    let mut compiled_files = Vec::new();

    std::thread::scope(|scope| {
        let mut threads = Vec::new();

        for (index, (partition, monitor)) in
            partitions.into_iter().zip(&config.monitors).enumerate()
        {
            // Spawn separate tasks for each physical display to optimize hardware efficiency
            let thread_handle = scope.spawn(move || -> WallSwitchResult<FileInfo> {
                compile_single_monitor_background(partition, monitor, config, index)
            });
            threads.push(thread_handle);
        }

        for handle in threads {
            let file_info = handle.join().unwrap()?;
            compiled_files.push(file_info);
        }

        Ok::<(), crate::WallSwitchError>(())
    })?;

    Ok(compiled_files)
}

/// Assembles multiple sub-images into a single cohesive canvas for a given monitor in-memory.
fn assemble_monitor_canvas(
    partition: &[FileInfo],
    monitor: &crate::Monitor,
) -> WallSwitchResult<RgbImage> {
    let mut monitor_canvas = RgbImage::new(
        monitor.resolution.width as u32,
        monitor.resolution.height as u32,
    );
    let target = LayoutTarget::calculate(monitor)?;

    let mut current_x = 0;
    let mut current_y = 0;

    for (p_idx, image_info) in partition.iter().enumerate() {
        let mut w = target.base_w;
        let mut h = target.base_h;

        match monitor.picture_orientation {
            Horizontal => {
                if p_idx < target.rem_h {
                    h += 1;
                }
            }
            Vertical => {
                if p_idx < target.rem_w {
                    w += 1;
                }
            }
        }

        // Load the image using the image crate
        let img = image::open(&image_info.path).map_err(|e| {
            WallSwitchError::UnableToFind(format!(
                "Failed to open image {:?}: {}",
                image_info.path, e
            ))
        })?;

        // Center crop and scale preserving aspect ratio (mimics magick -resize WxH^ -extent WxH)
        let resized = img
            .resize_to_fill(w as u32, h as u32, FilterType::Triangle)
            .to_rgb8();

        // Draw sub-image onto the monitor canvas
        image::imageops::overlay(
            &mut monitor_canvas,
            &resized,
            current_x as i64,
            current_y as i64,
        );

        // Adjust coordinates for the next image in the layout
        match monitor.picture_orientation {
            Horizontal => {
                current_y += h;
            }
            Vertical => {
                current_x += w;
            }
        }
    }

    Ok(monitor_canvas)
}

/// Stitches all compiled monitor canvases together to generate the final spanned multi-monitor wallpaper in-memory.
fn assemble_final_wallpaper(
    monitor_images: &[RgbImage],
    config: &Config,
) -> WallSwitchResult<RgbImage> {
    let mut total_w = 0;
    let mut total_h = 0;

    for monitor in &config.monitors {
        match config.monitor_orientation {
            Horizontal => {
                total_w += monitor.resolution.width;
                total_h = total_h.max(monitor.resolution.height);
            }
            Vertical => {
                total_w = total_w.max(monitor.resolution.width);
                total_h += monitor.resolution.height;
            }
        }
    }

    let mut final_canvas = RgbImage::new(total_w as u32, total_h as u32);
    let mut current_x = 0;
    let mut current_y = 0;

    for (idx, img) in monitor_images.iter().enumerate() {
        image::imageops::overlay(&mut final_canvas, img, current_x as i64, current_y as i64);

        match config.monitor_orientation {
            Horizontal => {
                current_x += config.monitors[idx].resolution.width;
            }
            Vertical => {
                current_y += config.monitors[idx].resolution.height;
            }
        }
    }

    Ok(final_canvas)
}

fn get_partitions_iter<'a>(
    mut images: &'a [FileInfo],
    config: &'a Config,
) -> impl Iterator<Item = &'a [FileInfo]> {
    config.monitors.iter().map(move |monitor| {
        let (head, tail) = images.split_at(monitor.pictures_per_monitor.into());
        images = tail;
        head
    })
}

pub fn exec_cmd(cmd: &mut Command, verbose: bool, msg: &str) -> WallSwitchResult<Output> {
    let output: Output = cmd.output().map_err(|e| {
        eprintln!("Failed to execute command: {:?}", cmd.get_program());
        WallSwitchError::Io(e)
    })?;

    let program = cmd.get_program();
    let arguments: Vec<_> = cmd.get_args().collect();

    if !output.status.success() || verbose {
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

        return Err(WallSwitchError::CommandFailed {
            program: format!("{:?}", cmd.get_program()),
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    Ok(output)
}
