#!/bin/bash

echoerr() { echo "$@" 1>&2; }

if [ -f "$HOME/dotfiles/lib/term.sh" ]; then
    source $HOME/dotfiles/lib/term.sh
else
    echoerr "Dotfiles not installed"
    exit 1
fi

print_header "Copying files from repository to dotfiles"
cp -v shell/fish/functions/* $HOME/dotfiles/fish/functions/
cp -v shell/fish/completions/* $HOME/dotfiles/fish/completions/

$HOME/dotfiles/fish/install.sh
