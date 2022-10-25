use crate::{cli, CommandDef, Config, Destination};
use clap::Shell;
use itertools::Itertools;
use std::{
    borrow::Cow,
    fs::File,
    path::{Path, PathBuf},
};

pub(crate) fn command_completions(sub_m: &clap::ArgMatches) {
    let shell = sub_m.value_of("shell").expect("shell is required argument");
    let for_shell = match shell {
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        x => {
            eprintln!("Unknown shell type: {}", x);
            std::process::exit(0);
        }
    };
    let bin_name = env!("CARGO_BIN_NAME");
    cli::build_cli().gen_completions_to(bin_name, for_shell, &mut std::io::stdout());
}

pub(crate) fn command_get(sub_m: &clap::ArgMatches, config: &Config) {
    let destination = sub_m
        .value_of("alias")
        .map(|alias| config.aliases.get(alias))
        .flatten();

    if let Some(destination) = destination {
        println!("{}", destination);
    } else {
        std::process::exit(1)
    }
}

pub(crate) fn command_set<P: AsRef<Path>>(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    let cwd = get_cwd_or_exit();
    let alias = {
        sub_m
            .value_of("alias")
            .map(|x| Cow::Borrowed(x))
            .unwrap_or_else(|| {
                let alias = cwd.file_name().map(|f| f.to_string_lossy());
                if let Some(inner) = alias {
                    inner
                } else {
                    eprintln!("Last part of CWD could not be retrieved");
                    std::process::exit(1);
                }
            })
    };
    let path = if let Some(_) = sub_m.value_of("remote") {
        if let Some(path) = sub_m.value_of("path") {
            Cow::Borrowed(path)
        } else {
            eprintln!("CWD cannot be used for --remote");
            std::process::exit(1);
        }
    } else if let Some(path) = sub_m.value_of("path") {
        Cow::Borrowed(path)
    } else {
        cwd.to_string_lossy()
    };
    if config.aliases.contains_key(alias.as_ref()) && !sub_m.is_present("force") {
        eprintln!("Alias already exists. Use -f to replace it anyway.");
        std::process::exit(1);
    }
    let value = if let Some(remote) = sub_m.value_of("remote") {
        Destination::Remote {
            remote: remote.to_owned(),
            path: path.to_string(),
        }
    } else {
        Destination::Local(path.to_string())
    };
    let _ = config.aliases.insert(alias.to_string(), value.clone());
    save_config_file(config_path, &*config)?;
    match &value {
        Destination::Local(path) => println!("{}\t{}", alias, path),
        Destination::Remote { remote, path } => println!("{}\t{}\t{}", alias, path, remote),
    }
    Ok(())
}

pub(crate) fn command_ls(sub_m: &clap::ArgMatches, config: &Config) {
    for alias in config.aliases.keys().sorted() {
        // for (alias, path) in &config.aliases {
        let path = &config.aliases[alias];
        match path {
            Destination::Local(_) => {
                if sub_m.is_present("remote_only") {
                    continue;
                }
            }
            Destination::Remote { path: _, remote: _ } => {
                if !sub_m.is_present("all") && !sub_m.is_present("remote_only") {
                    continue;
                }
            }
        }
        if sub_m.is_present("show_paths") {
            println!("{}\t{}", alias, path);
        } else {
            println!("{}", alias);
        }
    }
}

pub(crate) fn command_rm<P: AsRef<Path>>(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    let alias = sub_m.value_of("alias").expect("alias is required");
    if config.aliases.remove(alias).is_none() {
        std::process::exit(1);
    }
    save_config_file(config_path, &*config)?;
    Ok(())
}

pub(crate) fn command_mv(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &PathBuf,
) -> Result<(), anyhow::Error> {
    let alias_from = sub_m
        .value_of("alias_from")
        .expect("alias_from is required");
    let alias_to = sub_m
        .value_of("alias_to")
        .expect("alias_to is required argument");
    if config.aliases.contains_key(alias_to) {
        if !sub_m.is_present("force") {
            eprintln!("Destination alias exists. Use -f to replace it anyway.");
            std::process::exit(1);
        }
    }
    if let Some(path) = config.aliases.remove(alias_from) {
        config.aliases.insert(alias_to.to_owned(), path);
    } else {
        eprintln!("Alias not found: {}", alias_from);
        std::process::exit(1);
    }
    if let Some(prev_def) = config.commands.remove(alias_from) {
        config.commands.insert(alias_to.to_string(), prev_def);
    }
    save_config_file(config_path, &config)?;
    Ok(())
}

pub(crate) fn command_default<P: AsRef<Path>>(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    if sub_m.is_present("set") {
        let alias = sub_m.value_of("set").expect("required argument");
        if let Some(path) = config.aliases.get(alias) {
            config.default = Some(alias.to_owned());
            save_config_file(config_path, config)?;
            println!("{}\t{}", alias, path);
        } else {
            eprintln!("Alias not known.");
            std::process::exit(1);
        }
    } else if let Some(alias) = &config.default {
        if let Some(dest) = config.aliases.get(alias) {
            match dest {
                Destination::Local(path) => {
                    println!("{}\t{}", alias, path);
                }
                Destination::Remote { remote, path } => {
                    println!("{}\t{}\t{}", alias, path, remote);
                }
            }
        } else {
            eprintln!("Config error: default alias is not defined!");
            std::process::exit(1);
        }
    } else {
        eprintln!("Default alias not set");
        std::process::exit(1);
    }
    Ok(())
}

pub(crate) fn command_cmd<P: AsRef<Path>>(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    let alias = sub_m.value_of("alias").expect("alias is required");
    if let Some(dest) = config.aliases.get(alias) {
        if let Destination::Remote { remote: _, path: _ } = dest {
            eprintln!("Remote commands not supported");
            std::process::exit(1);
        }
    } else {
        eprintln!("Alias not found");
        std::process::exit(1);
    }
    if sub_m.is_present("set") {
        let cmd = sub_m.value_of("set").expect("set value is required");
        if let Some(def) = config.commands.get_mut(alias) {
            def.on_enter = cmd.to_owned();
        } else {
            let def = CommandDef {
                on_enter: cmd.to_owned(),
            };
            config.commands.insert(alias.to_owned(), def);
        }
        save_config_file(config_path, config)?
    } else if let Some(def) = config.commands.get(alias) {
        println!("{}", def.on_enter);
    }
    Ok(())
}

fn get_cwd_or_exit() -> PathBuf {
    match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("CWD cannot be used: path not found");
                }
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("CWD cannot be used: permission denied");
                }
                _ => {}
            };
            std::process::exit(1);
        }
    }
}

fn save_config_file<P: AsRef<Path>>(config_path: P, config: &Config) -> anyhow::Result<()> {
    let config_file = File::create(config_path)?;
    let _ = serde_json::to_writer_pretty(config_file, config)?;
    Ok(())
}
