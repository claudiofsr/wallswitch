complete -c wallswitch -s b -l min_size -d 'Set a minimum file size (in bytes) for searching image files' -r
complete -c wallswitch -s B -l max_size -d 'Set a maximum file size (in bytes) for searching image files' -r
complete -c wallswitch -s g -l generate -d 'Generate shell completions and exit the program' -r -f -a "bash\t''
elvish\t''
fish\t''
powershell\t''
zsh\t''"
complete -c wallswitch -s d -l min_dimension -d 'Set the minimum dimension that the height and width must satisfy' -r
complete -c wallswitch -s D -l max_dimension -d 'Set the maximum dimension that the height and width must satisfy' -r
complete -c wallswitch -s i -l interval -d 'Set the interval (in seconds) between each wallpaper displayed' -r
complete -c wallswitch -s m -l monitor -d 'Set the number of monitors [default: 2]' -r
complete -c wallswitch -s o -l orientation -d 'Inform monitor orientation: Horizontal (side-by-side) or Vertical (stacked)' -r
complete -c wallswitch -s p -l pictures_per_monitor -d 'Set number of pictures (or images) per monitor [default: 1]' -r
complete -c wallswitch -s c -l config -d 'Read the configuration file and exit the program'
complete -c wallswitch -s s -l sort -d 'Sort the images found'
complete -c wallswitch -s v -l verbose -d 'Show intermediate runtime messages'
complete -c wallswitch -s h -l help -d 'Print help (see more with \'--help\')'
complete -c wallswitch -s V -l version -d 'Print version'
