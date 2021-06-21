#!/usr/bin/fish

function __fish_j_complete_alias
    skoczek list -p
end

complete -f -c j -a "(__fish_j_complete_alias)"
