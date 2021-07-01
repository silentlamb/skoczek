#!/bin/bash

complete -W "$(skoczek ls -a | tr '\n' ' ')" j

if [ -f "$HOME/.skoczek/completions-gen.sh" ]; then
    source "$HOME/.skoczek/completions-gen.sh"
fi
