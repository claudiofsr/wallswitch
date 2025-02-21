# wallswitch

## Random Wallpaper for Multiple Monitors

```
#-----------#-----------# ... ... #-----------#
|           |           |         |           |
| Monitor 1 | Monitor 2 |         | Monitor n |
|           |           |         |           |
#-----------#-----------# ... ... #-----------#
```

**Description:**

`wallswitch` randomly selects wallpapers for multiple monitors.

**Features:**

* **Random Wallpaper Selection:** Dynamically chooses different wallpapers from designated directories for every monitor.

* **Configurable Filtering:**
    * **Dimension Control:** Refine your wallpaper choices by setting minimum and maximum dimensions (height and width) to include only images within a specific size range.

    * **File Size Management:** Specify minimum and maximum file sizes for searching, allowing you to exclude very large or small images. 

* **Flexible Configuration:** Customize `wallswitch` with options for:
    * **Directory Specification:** Define custom directories containing your wallpaper images using the  `directories` option.

    * **Image File Types:** Select specific image extensions (e.g., JPG, PNG, SVG) through the `extensions` option. 

    * **Resolution Matching:** Optimize display by matching wallpapers to monitor resolutions defined in the `resolutions` array.

* **Wallpaper Cycling Interval:** Set the time interval between wallpaper changes using the `interval` option.

* **Image Sorting:** Optionally sort selected images based on filename using the `sort` option. 

**Usage:**

Run the command:

```bash
wallswitch
```

The default number of monitors is set to `m = 2`.

Wallpapers will be recursively searched in the specified directories (`"directories": [...]`) for files with compatible extensions (`"extensions": [...]`).

## Configuration

The configuration file, located at `~/.config/wallswitch/wallswitch.json`, is key to customizing wallswitch behavior:

* **Displaying the Configuration:**
   - Direct Display: 
     ```bash
     wallswitch -c
     ```
   - Parsing with [jq](https://jqlang.github.io/jq/): 
     ```bash
     cat ~/.config/wallswitch/wallswitch.json | jq
     ```

The default configuration file has the following structure:

```json
{
  "desktop": "gnome", // Desktop environment (e.g., gnome, xfce4)
  "directories": [
    "/home/user_name/Figures",
    "/home/user_name/Images",
    "/home/user_name/Pictures",
    "/home/user_name/Wallpapers",
    "/home/user_name/Imagens",
    "/usr/share/wallpapers",
    "/usr/share/backgrounds",
    "/tmp/teste"
  ],
  "extensions": [
    "avif",
    "jpg",
    "jpeg",
    "png",
    "svg",
    "tif",
    "webp"
  ],
  "interval": 1800,        // Time interval (in seconds) between wallpaper changes
  "min_dimension": 600,    // Minimum dimension (width or height)
  "max_dimension": 128000, // Maximum dimension (width or height)
  "min_size": 1024,        // Minimum file size (in bytes)
  "max_size": 1073741824,  // Maximum file size (in bytes)
  "monitors": [
    {
      "picture_orientation": "Vertical", // Indicates how the pictures or images are combined
      "pictures_per_monitor": 3,         // Set number of pictures (or images) per monitor
      "resolution": {
        "width": 3840,
        "height": 2160
      }
    },
    {
      "picture_orientation": "Horizontal", // Indicates how the pictures or images are combined
      "pictures_per_monitor": 2,           // Set number of pictures (or images) per monitor
      "resolution": {
        "width": 3840,
        "height": 2160
      }
    }
  ],
  "monitor_orientation": "Horizontal",
  "path_feh": "/usr/bin/feh",
  "path_magick": "/usr/bin/magick", // magick or convert
  "sort": false,    // Sort the images found
  "verbose": false, // Show intermediate runtime messages
  "wallpaper": "/home/user_name/wallswitch.jpg"
}
```

**Set number of wallpaper per monitor (Gnome desktop only)**

You can display `N` different wallpapers on each monitor by using the `-p` flag followed by the desired number (`N`).

```bash
wallswitch -p N
```

The orientation of constructed images can be changed on each monitor between `Horizontal` and `Vertical`. 

## Wallpaper suggestions

* Get all gnome [backgrounds](https://github.com/zebreus/all-gnome-backgrounds/tree/master/data/images).

Dowload wallpapers with:

```
git clone https://github.com/zebreus/all-gnome-backgrounds.git
```

## Help messages

To get help messages, run the command:

```
wallswitch -h
```

The output:
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
  -i, --interval <INTERVAL>
          Set the interval (in seconds) between each wallpaper displayed
  -m, --monitor <MONITOR>
          Set the number of monitors [default: 2]
  -o, --orientation <MONITOR_ORIENTATION>
          Inform monitor orientation: Horizontal (side-by-side) or Vertical (stacked)
  -p, --pictures_per_monitor <PICTURES_PER_MONITOR>
          Set number of pictures (or images) per monitor [default: 1]
  -s, --sort
          Sort the images found
  -v, --verbose
          Show intermediate runtime messages
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

`min_dimension` is the minimum value obtained from the width and height of an image file:

`min_dimension = min(width, height)`.

`max_dimension` is the maximum value obtained from the width and height of an image file:

`max_dimension = max(width, height)`.

## Desktops

Make changes to the source code.

```
git clone https://github.com/claudiofsr/wallswitch.git

cd wallswitch
```

Edit the file: 'src/desktops.rs'.

To build and install from source, run the following command:

```
cargo b -r && cargo install --path=.
```

### gnome

Create a wallpaper file and set it as your desktop background image.

See the function:
```
fn set_gnome_wallpaper()
```

### xfce

See the function:

```
fn set_xfce_wallpaper()
```

Monitor 1: "/backdrop/screen0/monitorDP-0/workspace0/last-image"

Monitor 2: "/backdrop/screen0/monitorDP-2/workspace0/last-image"

Monitor N: ...

### openbox

See the function:

```
fn set_openbox_wallpaper()
```

## Mutually exclusive features

To use the [clap](https://crates.io/crates/clap) (default):
```
cargo b -r && cargo install --path=. --features args_v1
```

To reduce binary size, alternatively use my Command Line Argument Parser (args_v2.rs):
```
cargo b -r && cargo install --path=. --features args_v2
```

## Dependencies

* [imagemagick](https://imagemagick.org/) (image viewing/manipulation program).

* [feh](https://feh.finalrewind.org/) (fast and light image viewer).

