use crate::{
    cli::{CmdCommand, CmdDefault, CmdGet, CmdLs, CmdMv, CmdRm, CmdSet},
    CommandDef, Config, Destination,
};
use itertools::Itertools;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub(crate) fn command_get(args: CmdGet, config: &Config) {
    let destination = config.aliases.get(&args.alias);
    if let Some(destination) = destination {
        println!("{}", destination);
    } else {
        std::process::exit(1)
    }
}

pub(crate) fn command_set<P: AsRef<Path>>(
    args: CmdSet,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    let cwd = get_cwd_or_exit();
    let alias = args.alias.unwrap_or_else(|| {
        let alias = cwd.file_name().map(|f| f.to_string_lossy());
        if let Some(inner) = alias {
            inner.into_owned()
        } else {
            eprintln!("Last part of CWD could not be retrieved");
            std::process::exit(1);
        }
    });
    let path = if args.remote.is_some() {
        if let Some(path) = args.path {
            path
        } else {
            eprintln!("CWD cannot be used for --remote");
            std::process::exit(1);
        }
    } else {
        args.path
            .unwrap_or_else(|| cwd.to_string_lossy().into_owned())
    };
    if config.aliases.contains_key(&alias) && !args.force {
        eprintln!("Alias already exists. Use -f to replace it anyway.");
        std::process::exit(1);
    }
    let value = if let Some(remote) = args.remote {
        Destination::Remote { remote, path }
    } else {
        Destination::Local(path)
    };
    let _ = config.aliases.insert(alias.to_string(), value.clone());
    save_config_file(config_path, &*config)?;
    match &value {
        Destination::Local(path) => println!("{}\t{}", alias, path),
        Destination::Remote { remote, path } => println!("{}\t{}\t{}", alias, path, remote),
    }
    Ok(())
}

pub(crate) fn command_ls(args: CmdLs, config: &Config) {
    for alias in config.aliases.keys().sorted() {
        let path = &config.aliases[alias];
        match path {
            Destination::Local(_) => {
                if args.remote_only {
                    continue;
                }
            }
            Destination::Remote { path: _, remote: _ } => {
                if !args.all && !args.remote_only {
                    continue;
                }
            }
        }
        if args.show_paths {
            println!("{}\t{}", alias, path);
        } else {
            println!("{}", alias);
        }
    }
}

pub(crate) fn command_rm<P: AsRef<Path>>(
    args: CmdRm,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    if config.aliases.remove(&args.alias).is_none() {
        std::process::exit(1);
    }
    save_config_file(config_path, &*config)?;
    Ok(())
}

pub(crate) fn command_mv(
    args: CmdMv,
    config: &mut Config,
    config_path: &PathBuf,
) -> Result<(), anyhow::Error> {
    if config.aliases.contains_key(&args.alias_to) && !args.force {
        eprintln!("Destination alias exists. Use -f to replace it anyway.");
        std::process::exit(1);
    }
    if let Some(path) = config.aliases.remove(&args.alias_from) {
        config.aliases.insert(args.alias_to.to_owned(), path);
    } else {
        eprintln!("Alias not found: {}", args.alias_from);
        std::process::exit(1);
    }
    if let Some(prev_def) = config.commands.remove(&args.alias_from) {
        config.commands.insert(args.alias_to.to_string(), prev_def);
    }
    save_config_file(config_path, config)?;
    Ok(())
}

pub(crate) fn command_default<P: AsRef<Path>>(
    args: CmdDefault,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    if let Some(alias) = args.alias {
        if let Some(path) = config.aliases.get(&alias) {
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
    args: CmdCommand,
    config: &mut Config,
    config_path: P,
) -> Result<(), anyhow::Error> {
    if let Some(dest) = config.aliases.get(&args.alias) {
        if let Destination::Remote { remote: _, path: _ } = dest {
            eprintln!("Remote commands not supported");
            std::process::exit(1);
        }
    } else {
        eprintln!("Alias not found");
        std::process::exit(1);
    }
    if let Some(cmd) = args.command {
        if let Some(def) = config.commands.get_mut(&args.alias) {
            def.on_enter = cmd;
        } else {
            let def = CommandDef { on_enter: cmd };
            config.commands.insert(args.alias, def);
        }
        save_config_file(config_path, config)?
    } else if let Some(def) = config.commands.get(&args.alias) {
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
    serde_json::to_writer_pretty(config_file, config)?;
    Ok(())
}
