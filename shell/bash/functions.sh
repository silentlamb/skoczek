#!/bin/bash

function j {
    if ! [ -x "$(command -v skoczek)" ]; then
        echo "Command 'skoczek' cannot be find anywhere in a PATH" 1>&2
        return
    fi
    declare skoczek_project="$1"
    declare skoczek_path=""
    declare skoczek_remote=""
    declare on_enter_cmd=""

    if [ ! -z "${skoczek_project}" ]; then
        IFS=$'\t' read -r skoczek_path skoczek_remote <<< "$(skoczek get ${skoczek_project})"
    else
        IFS=$'\t' read -r skoczek_project skoczek_path skoczek_remote <<< "$(skoczek default)"
    fi
    if [ -z "${skoczek_path}" ]; then
        skoczek list -ap | sort | column -t
    else
        if [ -z "${skoczek_remote}" ]; then
            cd $skoczek_path
            on_enter_cmd=$(skoczek command $project)
            if [ ! -z "$on_enter_cmd" ]; then
                eval $on_enter_cmd
            fi
        else
            # TODO: Find a way to call $SHELL -i and *then* call eval $on_enter_cmd
            ssh -t ${skoczek_remote} "cd ${skoczek_path} && \$SHELL -i"
        fi
    fi
}
