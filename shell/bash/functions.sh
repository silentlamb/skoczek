#!/bin/bash

function j {
    if ! [ -x "$(command -v skoczek)" ]; then
        echo "Command 'skoczek' cannot be find anywhere in a PATH" 1>&2
        return
    fi
    declare project="$1"
    declare path=""
    declare remote=""

    if [ ! -z "${project}" ]; then
        IFS=$'\t' read -r path remote <<< "$(skoczek get ${project})"
    else
        IFS=$'\t' read -r path remote <<< "$(skoczek default)"
    fi
    if [ -z "${path}" ]; then
        skoczek list -ap | sort | column -t
    else
        if [ -z "${remote}" ]; then
            cd $path
        else
            ssh -t $remote "cd ${path} && \$SHELL -i"
        fi
    fi
}
