mod cli;
mod cmd;

use cli::{
    COMMAND_CMD, COMMAND_COMPLETIONS, COMMAND_DEFAULT, COMMAND_GET, COMMAND_LS, COMMAND_MV,
    COMMAND_RM, COMMAND_SET,
};
use cmd::{
    command_cmd, command_completions, command_default, command_get, command_ls, command_mv,
    command_rm, command_set,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
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
        Path::new(&home_path).join(".skoczek.json")
    };

    let app_m = cli::build_cli().get_matches();

    let config_path = match app_m.value_of("config") {
        Some(config) => match PathBuf::from_str(config) {
            Ok(config) => config,
            Err(_) => {
                eprintln!("Config path could not be parsed");
                std::process::exit(1);
            }
        },
        None => default_config_path,
    };
    let mut config = {
        let file = File::open(&config_path);
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
        (COMMAND_SET, Some(sub_m)) => command_set(sub_m, &mut config, &config_path)?,
        (COMMAND_RM, Some(sub_m)) => command_rm(sub_m, &mut config, &config_path)?,
        (COMMAND_MV, Some(sub_m)) => command_mv(sub_m, &mut config, &config_path)?,
        (COMMAND_DEFAULT, Some(sub_m)) => command_default(sub_m, &mut config, &config_path)?,
        (COMMAND_COMPLETIONS, Some(sub_m)) => command_completions(sub_m),
        (COMMAND_CMD, Some(sub_m)) => command_cmd(sub_m, &mut config, &config_path)?,
        _ => unreachable!(),
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
