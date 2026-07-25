use crate::{
    AwwwBackend, Colors, CommandExt, Config, Desktop, Dimension, Environment, FileInfo,
    HyprlandBackend, Monitor,
    Orientation::{Horizontal, Vertical},
    ProceduralEffect, SwaybgBackend, U8Extension, WallSwitchError, WallSwitchResult,
    detect_monitors, is_installed,
};
use image::{RgbImage, imageops::FilterType};
use rayon::prelude::*; // Required for parallel iterators
use std::{io::Error, process::Command};

/// Core trait defining the wallpaper application logic.
/// Follows the "Functional Core, Imperative Shell" pattern.
pub trait WallpaperBackend {
    /// PURE FUNCTION: Only constructs the required system commands.
    /// Defaults to returning an empty vector if not overridden.
    fn build_commands(_images: &[FileInfo], _config: &Config) -> WallSwitchResult<Vec<Command>> {
        Ok(vec![])
    }

    /// IMPURE FUNCTION: Executes the built commands.
    /// It defaults to sequentially running `build_commands`, but can be
    /// overridden by compositors that require complex state checks
    /// (e.g., Hyprland preloading, Swaybg daemon spawning).
    fn apply(images: &[FileInfo], config: &Config) -> WallSwitchResult<()> {
        let mut commands = Self::build_commands(images, config)?;
        for cmd in commands.iter_mut() {
            let program_name = cmd.get_program().to_string_lossy().to_string();
            // Using the new CommandExt trait for unified execution
            cmd.run_with_config(config, &format!("Executing {program_name}"))?;
        }
        Ok(())
    }
}

/// Set desktop wallpaper based on the detected Desktop Environment.
pub fn set_wallpaper(
    images: &[FileInfo],
    config: &Config,
    env: &Environment,
) -> WallSwitchResult<()> {
    // We ALWAYS compile wallpapers for all monitors.
    // This guarantees that:
    // 1. The output files are always lossless, highly compatible `.png` files in `/tmp`.
    // 2. Images are perfectly pre-cropped and pre-scaled to the native resolution of each monitor.
    // 3. The file paths passed to the backends are stable (/tmp/wallswitch_monitor_X.png),
    //    preventing VRAM leaks and file format errors (like WebP/AVIF unsupported by hyprpaper).
    let compiled_images = compile_wallpapers_for_monitors(images, config, env)?;

    // 2. Dispatch to the appropriate backend using the compiled single-image-per-monitor files
    match config.desktop {
        Desktop::Gnome => {
            if config.dry_run {
                println!(
                    "[DRY-RUN] Would stitch compiled monitor canvases together to generate final spanned wallpaper."
                );
            } else {
                // Memory optimized: Sequential loading to keep peak RSS low.
                let final_wallpaper = assemble_final_wallpaper(&compiled_images, config)?;
                final_wallpaper
                    .save(&config.wallpaper)
                    .map_err(|e| WallSwitchError::Io(Error::other(e)))?;

                if config.verbose {
                    println!("Stitched wallpaper saved to Gnome: {:?}", config.wallpaper);
                }
            }

            GnomeBackend::apply(&compiled_images, config)?;
        }

        Desktop::Xfce => XfceBackend::apply(&compiled_images, config)?,

        Desktop::Hyprland => {
            if is_installed("hyprpaper") {
                HyprlandBackend::apply(&compiled_images, config)?;
            } else if is_installed("awww") {
                AwwwBackend::apply(&compiled_images, config)?;
            } else if is_installed("swaybg") {
                SwaybgBackend::apply(&compiled_images, config)?;
            } else {
                return Err(WallSwitchError::MissingWaylandTools);
            }
        }

        Desktop::Niri | Desktop::Labwc | Desktop::Mango | Desktop::Wayland => {
            if is_installed("awww") {
                AwwwBackend::apply(&compiled_images, config)?;
            } else if is_installed("swaybg") {
                SwaybgBackend::apply(&compiled_images, config)?;
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
    /// Generates the GSettings commands required to set the GNOME desktop background.
    ///
    /// This method constructs the command execution vectors to update both `picture-uri`
    /// (light style) and `picture-uri-dark` (dark style) keys inside the
    /// `org.gnome.desktop.background` schema, as well as setting the rendering layout
    /// to `"spanned"`.
    ///
    /// # Errors
    ///
    /// This function does not currently return an error under standard operation,
    /// but returns a [`WallSwitchResult`] to comply with the [`WallpaperBackend`] trait.
    ///
    fn build_commands(_images: &[FileInfo], config: &Config) -> WallSwitchResult<Vec<Command>> {
        let mut commands = Vec::new();

        // Format the absolute file path into a standard "file://" URI
        let wallpaper_uri = format!("file://{}", config.wallpaper.display());

        // Construct commands to apply the URI to both light and dark theme backgrounds
        for picture_key in ["picture-uri", "picture-uri-dark"] {
            let mut cmd = Command::new("gsettings");
            cmd.args([
                "set",
                "org.gnome.desktop.background",
                picture_key,
                &wallpaper_uri,
            ]);
            commands.push(cmd);
        }

        // Construct command to set the picture options to spanned layout
        let mut span_cmd = Command::new("gsettings");
        span_cmd.args([
            "set",
            "org.gnome.desktop.background",
            "picture-options",
            "spanned",
        ]);
        commands.push(span_cmd);

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
    fn calculate(monitor: &Monitor) -> Result<Self, std::num::TryFromIntError> {
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
    monitor: &Monitor,
    config: &Config,
    index: usize,
) -> WallSwitchResult<()> {
    if config.effect == ProceduralEffect::None {
        return Ok(());
    }

    // 1. Resolve the effect once to prevent non-deterministic double-evaluation bugs
    let resolved = config.effect.resolve();

    // 2. Factory builds the resolved dynamic effect polymorphically (propagates Err if any)
    if let Some(renderer) = resolved.get_renderer(monitor, config)? {
        if config.verbose {
            let idx = index.to_string().bold().cyan();
            let name = resolved.get_name().bold().blue();

            // Dynamic dispatch prints the customized info of each concrete struct
            println!("Applying to Monitor {idx} {name} {}", renderer.info());
        }

        // Execute the render logic in-memory
        renderer.apply(canvas);
    }

    Ok(())
}

/// Compiles a single monitor canvas, applies overlays, saves the output to disk, and builds its FileInfo metadata.
fn compile_single_monitor_background(
    partition: &[FileInfo],
    monitor: &Monitor,
    config: &Config,
    env: &Environment,
    index: usize,
) -> WallSwitchResult<FileInfo> {
    let cache_dir = env.get_app_cache_dir();

    // Ensure the cache directory exists before writing to it
    if !config.dry_run {
        std::fs::create_dir_all(&cache_dir).map_err(WallSwitchError::Io)?;
    }

    let output_path = cache_dir.join(format!("wallswitch_monitor_{index}.png"));

    if config.dry_run {
        if config.verbose {
            println!(
                "[DRY-RUN] Would compile backgrounds for Monitor {index} at resolution {}x{}",
                monitor.resolution.width, monitor.resolution.height
            );
        }
    } else {
        // 1. Assemble separate pictures into a single composite monitor background in-memory
        let mut monitor_canvas = assemble_monitor_canvas(partition, monitor)?;

        // 2. Overlay dynamic procedural adjustments if any are requested
        if config.effect != ProceduralEffect::None {
            apply_selected_effect(&mut monitor_canvas, monitor, config, index)?;
        }

        // 3. Save compiled monitor canvas to disk
        monitor_canvas
            .save(&output_path)
            .map_err(|e| WallSwitchError::Io(Error::other(e)))?;

        if config.verbose {
            println!("Monitor {index} background assembled: {:?}", output_path);
        }
    }

    // 4. Construct structural metadata representing the updated target file
    Ok(FileInfo {
        path: output_path,
        size: 0,
        mtime: 0,
        hash: String::new(),
        dimension: Some(Dimension {
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
    env: &Environment,
) -> WallSwitchResult<Vec<FileInfo>> {
    if config.verbose {
        if config.dry_run {
            println!("[DRY-RUN] Would assemble multi-monitor wallpaper in pure Rust ...");
        } else {
            println!("Assembling multi-monitor wallpaper in pure Rust ...");
        }
    }

    // 1. First, collect the partitions into a Vec so we can use Rayon's parallel iterator.
    let partitions: Vec<&[FileInfo]> = get_partitions_iter(images, config).collect();

    // 2. Use Rayon to process the partitions in parallel.
    let compiled_files = partitions
        .into_par_iter()
        .zip(&config.monitors)
        .enumerate()
        .map(|(index, (partition, monitor))| {
            compile_single_monitor_background(partition, monitor, config, env, index)
        })
        .collect::<WallSwitchResult<Vec<_>>>()?;

    Ok(compiled_files)
}

/// Assembles multiple sub-images into a single cohesive canvas for a given monitor in-memory.
fn assemble_monitor_canvas(
    partition: &[FileInfo],
    monitor: &Monitor,
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

        // Memory optimization: Load, resize, and convert inside a nested block to drop
        // the heavy uncompressed DynamicImage (`img`) immediately before drawing.
        let resized = {
            // Load the image using the image crate
            let img =
                image::open(&image_info.path).map_err(|err| WallSwitchError::CorruptImage {
                    path: image_info.path.clone(),
                    source: err,
                })?;

            // Center crop and scale preserving aspect ratio
            img.resize_to_fill(w as u32, h as u32, FilterType::Triangle)
                .to_rgb8()
        };

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
    compiled_images: &[FileInfo],
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

    for (idx, img_info) in compiled_images.iter().enumerate() {
        // Load, convert to RGB8, draw, and immediately drop to keep memory consumption low
        let img = image::open(&img_info.path)
            .map_err(|e| {
                WallSwitchError::UnableToFind(format!(
                    "Failed to load compiled monitor canvas: {e}"
                ))
            })?
            .to_rgb8();

        image::imageops::overlay(&mut final_canvas, &img, current_x as i64, current_y as i64);

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
