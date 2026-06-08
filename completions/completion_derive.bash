_wallswitch() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="wallswitch"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        wallswitch)
            opts="-b -B -c -d -D -e -g -i -l -m -o -p -s -v -h -V --min_size --max_size --config --min_dimension --max_dimension --effect --effects-add-presets --effects-min-iterations --effects-max-iterations --generate --interval --list --monitor --orientation --once --pictures_per_monitor --sort --dry-run --transition-type --transition-duration --transition-fps --transition-angle --transition-pos --verbose --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --min_size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max_size)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -B)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --min_dimension)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max_dimension)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -D)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --effect)
                    COMPREPLY=($(compgen -W "none julia mandelbrot newton nova aurora star fractal polynomial random" -- "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -W "none julia mandelbrot newton nova aurora star fractal polynomial random" -- "${cur}"))
                    return 0
                    ;;
                --effects-add-presets)
                    COMPREPLY=($(compgen -W "true false" -- "${cur}"))
                    return 0
                    ;;
                --effects-min-iterations)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --effects-max-iterations)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --generate)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                -g)
                    COMPREPLY=($(compgen -W "bash elvish fish powershell zsh" -- "${cur}"))
                    return 0
                    ;;
                --interval)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --monitor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --orientation)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pictures_per_monitor)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transition-type)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transition-duration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transition-fps)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transition-angle)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --transition-pos)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _wallswitch -o nosort -o bashdefault -o default wallswitch
else
    complete -F _wallswitch -o bashdefault -o default wallswitch
fi
