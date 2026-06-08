#compdef wallswitch

autoload -U is-at-least

_wallswitch() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" : \
'-b+[Set a minimum file size (in bytes) for searching image files]:MIN_SIZE:_default' \
'--min_size=[Set a minimum file size (in bytes) for searching image files]:MIN_SIZE:_default' \
'-B+[Set a maximum file size (in bytes) for searching image files]:MAX_SIZE:_default' \
'--max_size=[Set a maximum file size (in bytes) for searching image files]:MAX_SIZE:_default' \
'-d+[Set the minimum dimension that the height and width must satisfy]:MIN_DIMENSION:_default' \
'--min_dimension=[Set the minimum dimension that the height and width must satisfy]:MIN_DIMENSION:_default' \
'-D+[Set the maximum dimension that the height and width must satisfy]:MAX_DIMENSION:_default' \
'--max_dimension=[Set the maximum dimension that the height and width must satisfy]:MAX_DIMENSION:_default' \
'-e+[Apply a procedural overlay effect to the selected wallpapers before displaying]:EFFECT:((none\:"No overlay effect is applied; displays the raw, unaltered wallpaper"
julia\:"Julia Set fractal overlay"
mandelbrot\:"Mandelbrot Set fractal overlay"
newton\:"Newton-Raphson Basin of Attraction fractal overlay"
nova\:"Nova Julia liquid fractal overlay"
aurora\:"Procedural Cosmic Aurora wave generator"
star\:"Procedural Starfield / Bokeh generator"
fractal\:"Fractal mode selector\: randomly chooses between Julia or Mandelbrot"
polynomial\:"Fractal mode selector\: randomly chooses between Newton or Nova"
random\:"Fully randomised mode selector\: picks any effect independently per display"))' \
'--effect=[Apply a procedural overlay effect to the selected wallpapers before displaying]:EFFECT:((none\:"No overlay effect is applied; displays the raw, unaltered wallpaper"
julia\:"Julia Set fractal overlay"
mandelbrot\:"Mandelbrot Set fractal overlay"
newton\:"Newton-Raphson Basin of Attraction fractal overlay"
nova\:"Nova Julia liquid fractal overlay"
aurora\:"Procedural Cosmic Aurora wave generator"
star\:"Procedural Starfield / Bokeh generator"
fractal\:"Fractal mode selector\: randomly chooses between Julia or Mandelbrot"
polynomial\:"Fractal mode selector\: randomly chooses between Newton or Nova"
random\:"Fully randomised mode selector\: picks any effect independently per display"))' \
'-g+[Generate shell completions and exit the program]:GENERATOR:(bash elvish fish powershell zsh)' \
'--generate=[Generate shell completions and exit the program]:GENERATOR:(bash elvish fish powershell zsh)' \
'-i+[Set the interval (in seconds) between each wallpaper displayed]:INTERVAL:_default' \
'--interval=[Set the interval (in seconds) between each wallpaper displayed]:INTERVAL:_default' \
'-l+[List all found images and exit]:CRITERIA:_default' \
'--list=[List all found images and exit]:CRITERIA:_default' \
'-m+[Set the number of monitors \[default\: 2\]]:MONITOR:_default' \
'--monitor=[Set the number of monitors \[default\: 2\]]:MONITOR:_default' \
'-o+[Inform monitor orientation\: Horizontal (side-by-side) or Vertical (stacked)]:MONITOR_ORIENTATION:_default' \
'--orientation=[Inform monitor orientation\: Horizontal (side-by-side) or Vertical (stacked)]:MONITOR_ORIENTATION:_default' \
'-p+[Set number of pictures (or images) per monitor \[default\: 1\]]:PICTURES_PER_MONITOR:_default' \
'--pictures_per_monitor=[Set number of pictures (or images) per monitor \[default\: 1\]]:PICTURES_PER_MONITOR:_default' \
'--transition-type=[Transition type for Wayland compositors using awww (e.g. wipe, wave, fade, random)]:TRANSITION_TYPE:_default' \
'--transition-duration=[Duration of the transition animation in seconds]:TRANSITION_DURATION:_default' \
'--transition-fps=[Frames per second for transition smoothness]:TRANSITION_FPS:_default' \
'--transition-angle=[Angle used by directional transitions (wipe, wave)]:TRANSITION_ANGLE:_default' \
'--transition-pos=[Origin position used by grow/outer transitions (e.g. center, top)]:TRANSITION_POS:_default' \
'-c[Read the configuration file and exit the program]' \
'--config[Read the configuration file and exit the program]' \
'--once[Run a single wallpaper update cycle and exit]' \
'-s[Sort the images found]' \
'--sort[Sort the images found]' \
'--dry-run[Run without applying the wallpapers (simulation mode)]' \
'-v[Show intermediate runtime messages]' \
'--verbose[Show intermediate runtime messages]' \
'-h[Print help (see more with '\''--help'\'')]' \
'--help[Print help (see more with '\''--help'\'')]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
}

(( $+functions[_wallswitch_commands] )) ||
_wallswitch_commands() {
    local commands; commands=()
    _describe -t commands 'wallswitch commands' commands "$@"
}

if [ "$funcstack[1]" = "_wallswitch" ]; then
    _wallswitch "$@"
else
    compdef _wallswitch wallswitch
fi
