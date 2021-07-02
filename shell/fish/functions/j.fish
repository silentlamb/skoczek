#!/usr/bin/fish


function j -d "Change current directory to default one (set by skoczek)"
    if not command -sq skoczek 
        echo "Command 'skoczek' cannot be find anywhere in a PATH"
        return
    end

    set skoczek_alias $argv[1]
    set skoczek_path
    set skoczek_remote

    if test -n "$skoczek_alias"
        set segments (string split \t (skoczek get $skoczek_alias 2>/dev/null))
        set skoczek_path $segments[1]
        set skoczek_remote $segments[2]
    end
    if test -z "$skoczek_path"
        set segments (string split \t (skoczek default))
        set skoczek_alias $segments[1]
        set skoczek_path $segments[2]
        set skoczek_remote $segments[3]
    end
    if test -n "$skoczek_path"
        if test -n "$skoczek_remote"
            ssh -t $skoczek_remote "cd $skoczek_path && \$SHELL -i"
        else
            cd $skoczek_path
            set on_enter_cmd (skoczek command $skoczek_alias)
            if test -n "$on_enter_cmd"
                eval "$on_enter_cmd"
            end
        end
    else
        skoczek list -ap | sort | column -t
    end
end
