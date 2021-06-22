#!/usr/bin/fish


function j -d "Change current directory to default one (set by skoczek)"
    if not command -sq skoczek 
        echo "Command 'skoczek' cannot be find anywhere in a PATH"
        return
    end

    set alias_to_jump $argv[1]
    if test -n "$alias_to_jump"
        set path_to_jump (skoczek get $alias_to_jump)
    end
    if test -z "$path_to_jump"
        set path_to_jump (skoczek default)
    end
    if test -n "$path_to_jump"    
        cd $path_to_jump
    else
        skoczek list -p | sort | column -t
    end
end

