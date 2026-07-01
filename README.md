
# wallswitch

Random Wallpaper for Multiple Monitors

### Example Wallpapers (with Julia Fractal Overlays)

Below are two examples of generated wallpapers after applying the procedural Julia set fractal overlay effect. Click on the thumbnails below to view the images in full resolution:

<table align="center" style="border: none; border-collapse: collapse; width: 100%;">
  <tr style="border: none;">
    <td align="center" style="border: none; padding: 5px; width: 50%;">
      <a href="examples/wallswitch_monitor_0.png" target="_blank">
        <img src="examples/wallswitch_monitor_0.png" alt="Julia Fractal Overlay Example 1 (Dark Teal)" style="width: 100%; border-radius: 4px; box-shadow: 0 4px 8px rgba(0,0,0,0.2);"/>
      </a>
      <br/>
      <em>Example 1 (Dark Teal)</em>
    </td>
    <td align="center" style="border: none; padding: 5px; width: 50%;">
      <a href="examples/wallswitch_monitor_1.png" target="_blank">
        <img src="examples/wallswitch_monitor_1.png" alt="Julia Fractal Overlay Example 2 (Light Blue)" style="width: 100%; border-radius: 4px; box-shadow: 0 4px 8px rgba(0,0,0,0.2);"/>
      </a>
      <br/>
      <em>Example 2 (Light Blue)</em>
    </td>
  </tr>
</table>

```
#-----------#-----------# ... ... #-----------#
|           |           |         |           |
| Monitor 1 | Monitor 2 |         | Monitor n |
|           |           |         |           |
#-----------#-----------# ... ... #-----------#
```

### Description

`wallswitch` randomly selects and processes wallpapers for multiple monitors.

It is designed to be fast, and lightweight, performing all image stitching, cropping, scaling, and fractal generation in-process using pure Rust.

### Features

* **Multi-Picture Composition**: Dynamically combines up to N different wallpapers per monitor across all supported desktop environments.
* **Smart Caching & Visual Deduplication**:
    * Uses BLAKE3 hashing to index files.
    * Automatically skips visual duplicates (same image, different filename).
    * Smart cache checks modification times (mtime) for instant startup.
* **Procedural Overlay Effects**: Adds customizable mathematical overlays over your wallpapers. Configured via `-e / --effect <none|julia|mandelbrot|newton|nova|star|aurora|fractal|random>`:
    * **Julia Sets (`julia`)**: Detailed, randomized 360-degree rotated fractals. Uses continuous potential smooth coloring to prevent color-banding, and contrast-preserving dynamic halo blending to keep shapes visible on both light and dark backgrounds.
      * *Generator function: `f(z) = z^2 + c`, where `c` is a fixed constant and the initial `z` varies.*
    * **Mandelbrot Set (`mandelbrot`)**: Renders structural details and high-period cardioid bulb swirls.
      * *Generator function: `z(n+1) = z(n)^2 + c`, where the initial `z` is zero and `c` varies.*
    * **Newton-Raphson Basins (`newton`)**: Renders geometric, kaleidoscope-like mandala structures representing root-finding convergence fields across complex space boundaries.
      * *Generator function: `z(n+1) = z(n) - lambda * f(z(n)) / f'(z(n))` on the polynomial `f(z) = z^p - 1`.*
    * **Nova Julia (`nova`)**: Generates flowing, fluid-like plumes resembling liquid mercury, cosmic nebulae, or dynamic plasma current paths.
      * *Generator function: `z(n+1) = z(n) - R * (z(n)^p - 1) / (p * z(n)^(p-1)) + c`.*
    * **Starfield / Bokeh (`star`)**: Projects glowing, circular stars and light orbs of varying sizes, intensities, and neon colors with smooth Gaussian light falloffs.
      * *Generator function: `I(d) = I_0 * exp(-d^2 / (2 * sigma^2))`.*
    * **Cosmic Aurora (`aurora`)**: Generates glowing atmospheric wave filaments using multi-frequency wave mathematics.
      * *Generator function: `alpha = 0.25 * (sin(d_u * x) + cos(d_v * y) + sin(d_w * x + rho) + cos(sqrt(u^2 + v^2) * d_w4))`.*
    * **Fractal Mode (`fractal`)**: Randomly selects between Julia or Mandelbrot fractal overlays for the cycle.
    * **Polynomial Mode (`polynomial`)**: Randomly selects between Newton-Raphson Basins or Nova Julia fractal overlays for the cycle.
    * **Randomized Mode (`random`)**: Automatically decides on a random overlay effect independently for each physical display.
* **Highly Optimized Parallel Processing**: Core rendering routines for procedural calculations and image stitching are fully parallelized across all logical CPU cores using standard library thread scopes, maximizing execution speed.
* **Configurable Filtering**:
    * Dimension Control: Filter images by minimum/maximum width and height.
    * File Size Management: Exclude images based on byte size.
* **Flexible Configuration**:
    * Custom directories and image extensions (AVIF, JPG, PNG, WEBP, TIF, etc.).
    * Monitor-specific settings (orientation and pictures per monitor).
* **Advanced Listing**:
    * Sort your entire collection by size, dimensions, aspect ratio, or date.

### Usage

Standard background loop:
```
wallswitch
```
Run once and exit (useful for login scripts or cron):
```
wallswitch --once
```
Test behavior without applying changes:
```
wallswitch --dry-run
```
Set N different wallpapers per monitor (All desktops):
```
wallswitch -p N
```
Apply a specific Julia Sets overlay on wallpapers:
```
wallswitch -e julia
```

### Configuration

The configuration file is located at:
```
  ~/.config/wallswitch/wallswitch.json
```

Displaying the Configuration:
```
wallswitch -c
```
The default configuration file structure:
```
{
  "desktop": "gnome",
  "directories": [
    "/home/user_name/Figures",
    "/home/user_name/Images",
    "/home/user_name/Pictures",
    "/home/user_name/Wallpapers",
    "/home/user_name/Imagens",
    "/usr/share/backgrounds"
  ],
  "extensions": [
    "avif",
    "jpg",
    "jpeg",
    "png",
    "tif",
    "webp"
  ],
  "interval": 1800,
  "min_dimension": 600,
  "max_dimension": 128000,
  "min_size": 1024,
  "max_size": 1073741824,
  "monitors": [
    {
      "picture_orientation": "Vertical",
      "pictures_per_monitor": 1,
      "resolution": {
        "width": 3840,
        "height": 2160
      }
    },
    {
      "picture_orientation": "Horizontal",
      "pictures_per_monitor": 1,
      "resolution": {
        "width": 3840,
        "height": 2160
      }
    }
  ],
  "monitor_orientation": "Horizontal",
  "path_feh": "/usr/bin/feh",
  "sort": false,
  "effect": "juliaset",
  "effects": {
    "add_presets": true,
    "min_iterations": 600,
    "max_iterations": 1200,
    "julia": [ ... ],
    "mandelbrot": [ ... ],
    "newton": [ ... ],
    "nova": [ ... ]
  },
  "wallpaper": "/home/user_name/.cache/wallswitch/wallswitch.png",
  "transition_type": "random",
  "transition_duration": 2,
  "transition_fps": 60,
  "transition_angle": 45,
  "transition_pos": "center"
}

```

### Listing and Sorting

List images using `--list <CRITERIA>`.

#### Table sorting options:
  * path: Sort by full system path.
  * name: Sort by filename only.
  * size: Sort by file size (ascending).
  * sizedesc: Sort by file size (descending).
  * width: Sort by image width.
  * height: Sort by image height.
  * area: Sort by total pixels (width x height).
  * ratio: Sort by aspect ratio (e.g., 16:9).
  * time: Sort by last modification date.

#### JSON state options:
  * processed: List probed images with dimension metadata (JSON).
  * unprocessed: List images pending dimension probing (JSON).
  * cache: Full dump of the metadata cache (JSON).

Example:
```
wallswitch --list ratio
```

### Wallpaper Suggestions

* Get all gnome backgrounds:
```
git clone https://github.com/zebreus/all-gnome-backgrounds.git
```

### Help Messages
```
Run: wallswitch -h
```
```
randomly selects wallpapers for multiple monitors
Usage: wallswitch [OPTIONS]

Options:
  -b, --min_size <MIN_SIZE>
          Set a minimum file size (in bytes) for searching image files
  -B, --max_size <MAX_SIZE>
          Set a maximum file size (in bytes) for searching image files
  -c, --config
          Read the configuration file and exit the program
  -d, --min_dimension <MIN_DIMENSION>
          Set the minimum dimension that the height and width must satisfy
  -D, --max_dimension <MAX_DIMENSION>
          Set the maximum dimension that the height and width must satisfy
  -e, --effect <EFFECT>
          Apply a procedural overlay effect to the selected wallpapers before displaying [possible values: none, julia, mandelbrot, newton, nova, star, aurora, fractal, polynomial, random]
  -g, --generate <GENERATOR>
          Generate shell completions and exit the program [possible values: bash, elvish, fish, powershell, zsh]
  -i, --interval <INTERVAL>
          Set the interval (in seconds) between each wallpaper displayed
  -l, --list <CRITERIA>
          List all found images and exit
  -m, --monitor <MONITOR>
          Set the number of monitors [default: 2]
  -o, --orientation <MONITOR_ORIENTATION>
          Inform monitor orientation: Horizontal (side-by-side) or Vertical (stacked)
      --once
          Run a single wallpaper update cycle and exit
  -p, --pictures_per_monitor <PICTURES_PER_MONITOR>
          Set number of pictures (or images) per monitor [default: 1]
  -s, --sort
          Sort the images found
      --dry-run
          Run without applying the wallpapers (simulation mode)
      --transition-type <TRANSITION_TYPE>
          Transition type for Wayland compositors using awww (e.g. wipe, wave, fade, random)
      --transition-duration <TRANSITION_DURATION>
          Duration of the transition animation in seconds
      --transition-fps <TRANSITION_FPS>
          Frames per second for transition smoothness
      --transition-angle <TRANSITION_ANGLE>
          Angle used by directional transitions (wipe, wave)
      --transition-pos <TRANSITION_POS>
          Origin position used by grow/outer transitions (e.g. center, top)
  -v, --verbose
          Show intermediate runtime messages
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version


Config file:
  /home/claudio/.config/wallswitch/wallswitch.json

Examples:
  # Start the automatic background loop using default settings
  wallswitch

  # Run a single wallpaper update cycle and exit (useful for cron jobs)
  wallswitch --once

  # Change wallpaper every 10 minutes (600 seconds)
  wallswitch --interval 600

  # Set 3 different wallpapers per monitor (Gnome desktop only)
  wallswitch --pictures_per_monitor 3

  # Filter images by dimension (min 1080px) and file size (max 5MB)
  wallswitch --min_dimension 1080 --max_size 5242880

  # Apply a specific Julia Sets fractal overlay on wallpapers
  wallswitch --effect julia

  # Apply random fractal overlays [julia, mandelbrot]
  wallswitch --effect fractal

  # Apply randomized procedural overlays (fractal, polynomial, stars, auroras) on wallpapers
  wallswitch --effect random

  # Dry run mode to see what would be executed without applying changes
  wallswitch --dry-run --verbose

  # Wayland (awww): Use specific transition effects and duration
  wallswitch --transition-type wave --transition-duration 3

  # List all found images sorted by file size
  wallswitch --list size

  # Display all processed images (with dimensions) in JSON format
  wallswitch --list processed

  # Display all images that haven't been probed yet
  wallswitch --list unprocessed

  # Count processed images using jq
  wallswitch -l processed | jq 'length'
```

### Installation and Desktops

Build from source:
```
git clone https://github.com/claudiofsr/wallswitch.git
cd wallswitch
cargo b -r && cargo install --path=.
```

Desktop Specifics:
  * Gnome    : Assembles composite backgrounds in memory, saves the final spanned file, and sets it via 'gsettings'.
  * XFCE     : Assembles composite backgrounds in memory, saves separate monitor backgrounds, and applies them via 'xfconf-query'.
  * Wayland  : Robust detection for Hyprland, Niri, Labwc, Mango.
               Assembles separate monitor backgrounds, and applies them.
               Backend priority: awww -> swaybg -> hyprpaper.
  * X11/Other: Fallback to 'feh'.

### Dependencies

* feh         : Fast viewer for X11/Openbox.
* awww        : Animated daemon for Wayland (highly recommended).
* swaybg      : Reliable static wallpaper tool for Wayland.
* hyprpaper   : Wallpaper utility for Hyprland users.

### License

Copyright (c) 2023, Claudio Fernandes de Souza Rodrigues.

All rights reserved.

Distributed under the BSD-3-Clause License.
