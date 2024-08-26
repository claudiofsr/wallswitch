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
'-b+[Set a minimum file size (in bytes) for searching image files]:MIN_SIZE: ' \
'--min_size=[Set a minimum file size (in bytes) for searching image files]:MIN_SIZE: ' \
'-B+[Set a maximum file size (in bytes) for searching image files]:MAX_SIZE: ' \
'--max_size=[Set a maximum file size (in bytes) for searching image files]:MAX_SIZE: ' \
'-g+[Generate shell completions and exit the program]:GENERATOR:(bash elvish fish powershell zsh)' \
'--generate=[Generate shell completions and exit the program]:GENERATOR:(bash elvish fish powershell zsh)' \
'-d+[Set the minimum dimension that the height and width must satisfy]:MIN_DIMENSION: ' \
'--min_dimension=[Set the minimum dimension that the height and width must satisfy]:MIN_DIMENSION: ' \
'-D+[Set the maximum dimension that the height and width must satisfy]:MAX_DIMENSION: ' \
'--max_dimension=[Set the maximum dimension that the height and width must satisfy]:MAX_DIMENSION: ' \
'-i+[Set the interval (in seconds) between each wallpaper displayed]:INTERVAL: ' \
'--interval=[Set the interval (in seconds) between each wallpaper displayed]:INTERVAL: ' \
'-m+[Set the number of monitors \[default\: 2\]]:MONITOR: ' \
'--monitor=[Set the number of monitors \[default\: 2\]]:MONITOR: ' \
'-o+[Inform monitor orientation\: Horizontal (side-by-side) or Vertical (stacked)]:MONITOR_ORIENTATION: ' \
'--orientation=[Inform monitor orientation\: Horizontal (side-by-side) or Vertical (stacked)]:MONITOR_ORIENTATION: ' \
'-p+[Set number of pictures (or images) per monitor \[default\: 1\]]:PICTURES_PER_MONITOR: ' \
'--pictures_per_monitor=[Set number of pictures (or images) per monitor \[default\: 1\]]:PICTURES_PER_MONITOR: ' \
'-c[Read the configuration file and exit the program]' \
'--config[Read the configuration file and exit the program]' \
'-s[Sort the images found]' \
'--sort[Sort the images found]' \
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
