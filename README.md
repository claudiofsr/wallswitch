# wallswitch

```
#-----------#-----------# ... ... #-----------#
|           |           |         |           |
| Monitor 1 | Monitor 2 |         | Monitor n |
|           |           |         |           |
#-----------#-----------# ... ... #-----------#
```
Randomly sets wallpapers for `n` Linux desktop monitors (arranged horizontally).

[Define papéis de parede aleatoriamente para desktop Linux com `n` monitores (dispostos horizontalmente).]

`wallswitch` randomly selects and applies different wallpapers to each of n monitors arranged horizontally.

**Features:**

* **Random Wallpaper Selection:** Automatically chooses different wallpapers from your specified directories for each monitor. 
* **Duplicate Avoidance:** Ignores identical image files to ensure unique wallpapers on each screen.
* **Configurable Filtering:**

   -  **Image Size (Dimension):** Filter images based on minimum and maximum dimension (height, width).
   -  **File Size:** Specify minimum and maximum file sizes for searching. 

**Usage:**

Run the command:

```bash
wallswitch
```

The default number of monitors is `n` = 2.

Images are recursively searched in the indicated directories (`"dirs": [...]`) for files with extensions ["avif", "jpg", "jpeg", "png", "svg", "tif", "webp"].

## Help messages

To get help messages, run the command:

```
wallswitch -h
```

The output:
```
randomly sets wallpapers for n Linux desktop monitors (arranged horizontally).

Usage: wallswitch [OPTIONS]

Options:
  -c, --config
          Read the configuration file and exit the program
  -d, --min_dimension <MIN_DIMENSION>
          Set the minimum dimension that the height and width must satisfy
  -D, --max_dimension <MAX_DIMENSION>
          Set the maximum dimension that the height and width must satisfy
  -b, --min_size <MIN_SIZE>
          Set a minimum file size (in bytes) for searching image files
  -B, --max_size <MAX_SIZE>
          Set a maximum file size (in bytes) for searching image files
  -i, --interval <INTERVAL>
          Set the interval (in seconds) between each wallpaper displayed
  -n, --monitor <MONITOR>
          Set the number of monitors [default: 2]
  -s, --sort
          Sort the images found
  -v, --verbose
          Show intermediate runtime messages
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

`min_dimension` is the minimum value obtained from the height and width of an image file:

`min_dimension = min(height, width)`.

## Configuration

**Displaying the Configuration**

You can view the contents of the wallswitch configuration file using these commands:

* Displays the configuration directly:
```
wallswitch -c
```
* Parses the `json` configuration using the [jq](https://jqlang.github.io/jq/) command-line tool (requires `jq` to be installed):
```
cat ~/.config/wallswitch/wallswitch.json | jq
```

The default configuration file (`~/.config/wallswitch/wallswitch.json`) has the following structure:

```json
{
  "desktop": "gnome",
  "min_dimension": 600,
  "max_dimension": 128000,
  "min_size": 1024,
  "max_size": 1073741824,
  "dirs": [
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
  "interval": 1800,
  "monitor": 2,
  "sort": false,
  "verbose": false,
  "wallpaper": "/home/user_name/wallswitch.jpg"
}

```

**Editing the Configuration**

To customize wallswitch behavior, edit the configuration file directly:

* **Directory paths:** Modify the "dirs" array to include locations containing your desired image files. Remember to adjust the path according to your user name.

* **Dimensions:** Choose the minimum and maximum dimensions.

* **Extensions:** Modify the “extensions” array to select the image file extensions.

* **Interval:** Change the "interval" value (in seconds) to control how often wallpapers are cycled.

**Important Note:** Ensure that all directories listed in "dirs" have read permissions for the user running wallswitch.

## Wallpaper suggestions

* Get all gnome [backgrounds](https://github.com/zebreus/all-gnome-backgrounds/tree/master/data/images).

Dowload wallpapers with:

```
git clone https://github.com/zebreus/all-gnome-backgrounds.git
```

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

