mod cli;

use clap::Shell;
use cli::{
    COMMAND_CMD, COMMAND_COMPLETIONS, COMMAND_DEFAULT, COMMAND_GET, COMMAND_LS, COMMAND_MV,
    COMMAND_RM, COMMAND_SET,
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File, path::Path};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Destination {
    Local(String),
    Remote { remote: String, path: String },
}

impl std::fmt::Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Destination::Local(path) => write!(f, "{}", path),
            Destination::Remote { remote, path } => write!(f, "{}\t{}", path, remote),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandDef {
    pub on_enter: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, Destination>,

    #[serde(default)]
    pub default: Option<String>,

    #[serde(default)]
    pub commands: HashMap<String, CommandDef>,
}

fn run() -> anyhow::Result<()> {
    let default_config_path = {
        // TODO: Fix default path
        let home_path = std::env::var("HOME")?;
        let config_path = Path::new(&home_path)
            .join(".skoczek.json")
            .into_os_string()
            .into_string()
            .expect("into_string");
        config_path
    };

    let app_m = cli::build_cli().get_matches();

    let config_path = match app_m.value_of("config") {
        Some(config) => config,
        None => default_config_path.as_str(),
    };
    let mut config = {
        let file = File::open(config_path);
        if let Err(err) = &file {
            match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => {
                    eprintln!("error: {}", err);
                    std::process::exit(1);
                }
            }
        }
        if let Ok(inner_file) = file {
            serde_json::from_reader(inner_file)?
        } else {
            Config::default()
        }
    };

    match app_m.subcommand() {
        (COMMAND_GET, Some(sub_m)) => command_get(sub_m, &config),
        (COMMAND_LS, Some(sub_m)) => command_ls(sub_m, &config),
        (COMMAND_SET, Some(sub_m)) => command_set(sub_m, &mut config, config_path)?,
        (COMMAND_RM, Some(sub_m)) => command_rm(sub_m, &mut config, config_path)?,
        (COMMAND_MV, Some(sub_m)) => command_mv(sub_m, &mut config, config_path)?,
        (COMMAND_DEFAULT, Some(sub_m)) => command_default(sub_m, &mut config, config_path)?,
        (COMMAND_COMPLETIONS, Some(sub_m)) => command_completions(sub_m),
        (COMMAND_CMD, Some(sub_m)) => command_cmd(sub_m, &mut config, config_path)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn command_completions(sub_m: &clap::ArgMatches) {
    let for_shell = match sub_m.value_of("shell").expect("shell is required argument") {
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        _ => {
            eprint!("Unknown shell type");
            std::process::exit(0);
        }
    };
    let bin_name = env!("CARGO_BIN_NAME");
    cli::build_cli().gen_completions_to(bin_name, for_shell, &mut io::stdout());
}

fn command_get(sub_m: &clap::ArgMatches, config: &Config) {
    let alias = sub_m.value_of("alias").expect("alias is required");
    match config.aliases.get(alias) {
        Some(path) => {
            println!("{}", path);
        }
        None => {
            std::process::exit(1);
        }
    }
}

fn get_cwd_or_exit() -> PathBuf {
    match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::NotFound => {
                    eprintln!("CWD cannot be used: path not found");
                },
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("CWD cannot be used: permission denied");
                },
                _ => {}
            };
            std::process::exit(1);
        }
    }
}

fn command_set(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    let cwd = get_cwd_or_exit();
    let alias = {
        match sub_m.value_of("alias") {
            Some(x) => x.to_owned(),
            None => match cwd.file_name() {
                Some(name) => name.to_string_lossy().into_owned(),
                None => {
                    eprintln!("Last part of CWD could not be retrieved");
                    std::process::exit(1);
                },
            },
        }
    };
    let path = match sub_m.value_of("remote") {
        Some(_) => match sub_m.value_of("path") {
            Some(path) => path.to_owned(),
            None => {
                eprintln!("CWD cannot be used for --remote");
                std::process::exit(1);
            }
        },
        None => match sub_m.value_of("path") {
            Some(path) => path.to_owned(),
            None => cwd.to_string_lossy().into_owned(),
        }
    };

    if config.aliases.contains_key(&alias) && !sub_m.is_present("force") {
        eprintln!("Alias already exists. Use -f to replace it anyway.");
        std::process::exit(1);
    }

    let value = match sub_m.value_of("remote") {
        Some(remote) => Destination::Remote {
            remote: remote.to_owned(),
            path: path.clone(),
        },
        None => Destination::Local(path.clone()),
    };

    let _ = config.aliases.insert(alias.clone(), value.clone());
    save_config_file(config_path, &*config)?;
    match &value {
        Destination::Local(path) => println!("{}\t{}", alias, path),
        Destination::Remote { remote, path } => println!("{}\t{}\t{}", alias, path, remote),
    }
    Ok(())
}

fn command_ls(sub_m: &clap::ArgMatches, config: &Config) {
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

fn command_rm(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    let alias = sub_m.value_of("alias").expect("alias is required");
    if config.aliases.remove(alias).is_none() {
        std::process::exit(1);
    }
    save_config_file(config_path, &*config)?;
    Ok(())
}

fn command_mv(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    let alias_from = sub_m
        .value_of("alias_from")
        .expect("alias_from is required");
    let alias_to = sub_m
        .value_of("alias_to")
        .expect("alias_to is required argument")
        .to_owned();
    if config.aliases.contains_key(&alias_to) {
        if !sub_m.is_present("force") {
            eprintln!("Destination alias exists. Use -f to replace it anyway.");
            std::process::exit(1);
        }
    }
    match config.aliases.remove(alias_from) {
        Some(ref path) => {
            config.aliases.insert(alias_to.clone(), path.clone());
        }
        None => {
            eprintln!("Alias not found: {}", alias_from);
            std::process::exit(1);
        }
    }
    if let Some(prev_def) = config.commands.remove(alias_from) {
        config.commands.insert(alias_to, prev_def);
    }

    save_config_file(config_path, &config)?;

    Ok(())
}

fn command_default(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    if sub_m.is_present("set") {
        let alias = sub_m.value_of("set").expect("required argument");
        match config.aliases.get(alias) {
            Some(path) => {
                config.default = Some(alias.to_owned());
                save_config_file(config_path, config)?;
                println!("{}\t{}", alias, path);
            }
            None => {
                eprintln!("Alias not known.");
                std::process::exit(1);
            }
        }
    } else {
        match &config.default {
            Some(alias) => match config.aliases.get(alias) {
                Some(dest) => match dest {
                    Destination::Local(path) => {
                        println!("{}\t{}", alias, path);
                    }
                    Destination::Remote { remote, path } => {
                        println!("{}\t{}\t{}", alias, path, remote);
                    }
                },
                None => {
                    eprintln!("Config error: default alias is not defined!");
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("Default alias not set");
                std::process::exit(1);
            }
        }
    }
    Ok(())
}

fn command_cmd(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    let alias = sub_m.value_of("alias").expect("alias is required");
    match config.aliases.get(alias) {
        Some(dest) => {
            if let Destination::Remote { remote: _, path: _ } = dest {
                eprintln!("Remote commands not supported");
                std::process::exit(1);
            }
        }
        None => {
            eprintln!("Alias not found");
            std::process::exit(1);
        }
    }
    if sub_m.is_present("set") {
        let cmd = sub_m.value_of("set").expect("set value is required");
        match config.commands.get_mut(alias) {
            Some(def) => {
                def.on_enter = cmd.to_owned();
            }
            None => {
                let def = CommandDef {
                    on_enter: cmd.to_owned(),
                };
                config.commands.insert(alias.to_owned(), def);
            }
        }
        save_config_file(config_path, config)?
    } else {
        if let Some(def) = config.commands.get(alias) {
            println!("{}", def.on_enter);
        }
    }
    Ok(())
}

fn save_config_file(config_path: &str, config: &Config) -> anyhow::Result<()> {
    let config_file = File::create(config_path)?;
    let _ = serde_json::to_writer_pretty(config_file, config)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
