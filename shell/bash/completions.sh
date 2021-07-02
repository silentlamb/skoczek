#!/bin/bash

_skoczek_list_aliases() {
    skoczek ls -a | paste -s -d ' '
}


#
# Note: this completion function is based on the output of:
#       skoczek completion bash
#
# However it was slightly modified to be more aware of arguments (aliases, hostnames)
#
_skoczek() {
    local i cur prev opts cmds aliases contains_alias
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd="skoczek"
    opts=""
    aliases=""
    aliases="$(_skoczek_list_aliases)"
    contains_alias="0"

    for i in ${COMP_WORDS[@]:1}
    do
        case "${i}" in
            command)
                cmd+="__command"
                ;;
            completions)
                cmd+="__completions"
                ;;
            default)
                cmd+="__default"
                ;;
            get)
                cmd+="__get"
                ;;
            help)
                cmd+="__help"
                ;;
            list)
                cmd+="__ls"
                ;;
            ls)
                cmd+="__ls"
                ;;
            mv)
                cmd+="__mv"
                ;;
            rm)
                cmd+="__rm"
                ;;
            set)
                cmd+="__set"
                ;;
            *)
                ;;
        esac
    done

    # Detect if any of extra arguments (after command and subcommand) matches an alias name
    for i in ${COMP_WORDS[@]:2}; do 
        if [[ " ${aliases} " =~ " ${i} " ]]; then
            contains_alias="1"
        fi
    done

    case "${cmd}" in
        skoczek)
            opts="-h -V -c --help --version --config set ls rm get mv default completions help list command"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --config)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
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
        skoczek__command)
            opts=" -h -V -s --help --version --set"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts} ${aliases}" -- "${cur}") )
                return 0
            fi

            return 0
            ;;
        skoczek__completions)
            opts=" -h -V --help --version bash fish"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --set)
                    COMPREPLY=()
                    return 0
                    ;;
                -s)
                    COMPREPLY=()
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            if [[ "${contains_alias}" == "0" ]]; then
                opts="${opts} ${aliases}"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__default)
            opts=" -h -V -s --help --version --set"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --set)
                    COMPREPLY=($(compgen -W "${aliases}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "${aliases}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__get)
            opts="-h -V --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts} ${aliases}" -- "${cur}") )
                return 0
            fi
            if [[ "${contains_alias}" == "0" ]]; then
                opts="${opts} ${aliases}"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__help)
            opts="-h -V --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__ls)
            opts="-p -a -r -h -V --show-paths --all --remote --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__mv)
            opts="-f -h -V --force --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts} ${aliases}" -- "${cur}") )
                return 0
            fi
            if [[ "${contains_alias}" == "0" ]]; then
                opts="${opts} ${aliases}"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__rm)
            opts="-h -V --help --version"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts} ${aliases}" -- "${cur}") )
                return 0
            fi
            if [[ "${contains_alias}" == "0" ]]; then
                opts="${opts} ${aliases}"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        skoczek__set)
            opts="-f -h -V -r --force --help --version --remote"
            if [[ ${cur} == -* ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --remote)
                    opts="$(awk '/^Host / {print $2}' ~/.ssh/config | paste -s -d ' ')"
                    COMPREPLY=($(compgen -W "${opts}" -- "${cur}") $(compgen -A hostname -- "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -W "${opts}" -- "${cur}") $(compgen -A hostname -- "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            if [[ "${contains_alias}" == "0" ]]; then
                opts="${opts} ${aliases}"
            fi
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") $(compgen -f "${cur}"))
            return 0
            ;;
    esac
}

complete -W "$(_skoczek_list_aliases)" j
complete -F _skoczek -o bashdefault -o default skoczek
