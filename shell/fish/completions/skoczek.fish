#!/usr/bin/fish

function skoczek_available_aliases
    skoczek list -p 
end


set -l skoczek_commands default get help ls list mv rm set -h --help -V --version -c --config

complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a default -d 'Get/set default alias'
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a get -d "Display path for a given alias"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a help -d "Prints help message"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a "ls list" -d "Displays known aliases and their paths"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a mv -d "Renames an alias"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a rm -d "Removes an alias"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a set -d "Assigns alias to a path"
complete -f -c skoczek -n "not __fish_seen_subcommand_from $skoczek_commands" -a "-c --config" -d "Sets custom config file"



# Default subcommand
set -l skoczek_default_args -s --set -h --help
complete -f -c skoczek -n "__fish_seen_subcommand_from default; and not __fish_seen_subcommand_from $skoczek_default_args" -a "$skoczek_default_args" -d "Sets default alias"
complete -f -c skoczek -n "__fish_seen_subcommand_from default; and __fish_seen_subcommand_from -s --set; and not __fish_seen_subcommand_from (skoczek_available_aliases)" -a "(skoczek_available_aliases)"

# Get subcommand
complete -f -c skoczek -n "__fish_seen_subcommand_from get; and not __fish_seen_subcommand_from (skoczek_available_aliases)" -a "(skoczek_available_aliases)"

# Ls subcommand
set -l skoczek_ls_args -p --show-paths -h --help
complete -f -c skoczek -n "__fish_seen_subcommand_from ls list; and not __fish_seen_subcommand_from $skoczek_ls_args" -a "$skoczek_ls_args"

# Mv subcommand
function __skoczek_two_existing_aliases_given
    set -l aliases (skoczek list)
    set -l cmd (commandline -poc)
    set -e cmd[1]
    for i in (seq (count $cmd))
        echo "skoczek_two_existing_aliases_given: $i -> $cmd[$i]"
        if contains $cmd[$i] $aliases; and contains $cmd[(math $i + 1)] $aliases
            return 0
        end
    end
    return 1
end

complete -f -c skoczek -n "__fish_seen_subcommand_from mv; and not __skoczek_two_existing_aliases_given" -a "(skoczek_available_aliases)"
complete -f -c skoczek -n "__fish_seen_subcommand_from mv; and __skoczek_two_existing_aliases_given; and not __fish_seen_subcommand_from -f --force" -a "-f --force"

# Rm
complete -f -c skoczek -n "__fish_seen_subcommand_from rm; and not __fish_seen_subcommand_from (skoczek_available_aliases)" -a "(skoczek_available_aliases)"

# Set
complete -f -c skoczek -n "__fish_seen_subcommand_from set; and not __fish_seen_subcommand_from -f --force" -a "-f --force"
