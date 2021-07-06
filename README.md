# Skoczek

Command line utility to bookmark interesting paths in filesystems. The tool allows to assign a short name (alias) to the path, and query for paths using aliases.

This tool is made to be used with shell aliases/functions. Currently `bash` and `fish` shells are supported.

## Table of Contents

- [Skoczek](#skoczek)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
    - [Build sources](#build-sources)
    - [Install shell scripts](#install-shell-scripts)
      - [Fish shell](#fish-shell)
      - [Bash shell](#bash-shell)
  - [Usage](#usage)
  - [Remote paths](#remote-paths)
  - [Other features](#other-features)
  - [Licence](#licence)


## Installation

You can either use binaries from [Release page](https://github.com/silentlamb/skoczek/releases) or build it from the source code.

### Build sources

Requirements:

- **cargo**: https://www.rust-lang.org/tools/install


To build and install binary:

```
$ cargo build --release
$ cp target/release/skoczek $HOME/.local/bin
```

Where `$HOME/.local/bin` is a path visible in your `PATH`.

### Install shell scripts

Currently there are two shells supported: `fish` and `bash`.

#### Fish shell

To install:

```bash
mkdir -p config/fish/completions
mkdir -p config/fish/functions
cp shell/fish/completions/*.fish ~/.config/fish/completions
cp shell/fish/functions/*.fish ~/.config/fish/functions
```

This should make:

- `j` function to be available
- `j` arguments completion to be available
- `skoczek` arguments completion to be available

![Fish j completion](term02.png)

![Fish skoczek completion](term01.png)


#### Bash shell

To install:

```bash
mkdir -p $HOME/.skoczek
cp shell/bash/*.sh $HOME/.skoczek/
```

Then add the following lines somewhere in `~/.bashrc` or other file loaded by `bash` when starting:

```bash
if [ -f ~/.skoczek/functions.sh ]; then
    source ~/.skoczek/functions.sh
fi
if [ -f ~/.skoczek/functions.sh ]; then
    source ~/.skoczek/completions.sh
fi
```

## Usage

Typical workflow is as follow:

1. Assign a name to a directory
2. Assign optional "on enter" commands
3. Make that directory a default one
4. Go to the directory using its label

Commands:

```bash
# Step 1a: Assign label 'gapik' to a directory '~/Dev/Rust/gapik' 
skoczek set gapik ~/Dev/Rust/gapik

# Step 1b: Assign label 'gapik' to a current directory
~/Dev/Rust/gapik
skoczek set gapik

# Step 2: Add "on enter" commands
skoczek command gapik -s "clear; cat TODO.md"

# Step 3: Make 'gapik' a default project
skoczek default gapik

# Step 4a: Go to directory with a label 'gapik'
j gapik
# executed commands:
#   cd ~/Dev/Rust/gapik
#   clear
#   cat TODO.md

# Step 4b: Or go to directory behind a default label
j
```

Some other commands:

**List labels with paths:**

```
$ skoczek ls -p | column -t
actix     /home/marcin/Dev/Rust/actix-hello
dareczek  /home/marcin/DareczekViking
dotfiles  /home/marcin/dotfiles
gapik     /home/marcin/Dev/Rust/gapik
home      /home/marcin
skoczek   /home/marcin/Dev/Rust/skoczek
```

**Get path of a label:**

```
$ skoczek get gapik
/home/marcin/Dev/Rust/gapik
```

## Remote paths

Skoczek is also able to store paths to remote hosts:

```bash
# Step 1: Assign label 'malinka.dev' to '/home/ubuntu/dev' on host 'malinka'
skoczek set malinka.dev /home/ubuntu/dev -r malinka
# Note: malinka must be a name available to ssh (so ssh malinka connects to a host)

# Step 4: SSH to a path on remote machine behind label 'malinka.dev'
j malinka.dev
# executed commands:
#    ssh -t malinka "cd /home/ubuntu/dev && \$SHELL -i"
```

Currently "on enter" commands are not supported by remote paths - mostly because I couldn't
find a proper way to call commands like `source venv/bin/active` on remote machine without venv not being dropped after `$SHELL -i` call.

**To get list of remote paths:**

```bash
# Only remote paths:
$ skoczek ls -pr | sort | column -t
malinka.dev   /home/ubuntu/dev  malinka
malinka.home  /home/ubuntu      malinka

# Both remote and local paths:
$  skoczek ls -ap | sort | column -t
actix         /home/marcin/Dev/Rust/actix-hello
dareczek      /home/marcin/DareczekViking
dotfiles      /home/marcin/dotfiles
gapik         /home/marcin/Dev/Rust/gapik
home          /home/marcin
malinka.dev   /home/ubuntu/dev                   malinka
malinka.home  /home/ubuntu                       malinka
skoczek       /home/marcin/Dev/Rust/skoczek
```

## Other features

To get more features, run help command:

```
skoczek help
```

To get options for each sub-commands, call either:

```
skoczek help subcommand
skoczek subcommand -h
```


## Licence

[MIT License](LICENSE)
