#!/usr/bin/fish

function j -d "Change current directory to the one under the specified alias"
    if not command -sq skoczek 
        echo "Command 'skoczek' cannot be find anywhere in a PATH"
        return
    end
    if test -n "$argv[1]"
        set -l path_to_jump (skoczek get $argv[1])
        if test -n "$path_to_jump"
            cd $path_to_jump
        else
            echo "Path for alias '$argv[1]' is not known"
            return 1
        end
    else
        skoczek ls
    end
end
