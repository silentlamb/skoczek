mod cli;
mod cmd;
use clap::Parser;
use serde::{Deserialize, Serialize};
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
        // TODO: Fix default path - ????????????????????????????
        let home_path = std::env::var("HOME")?;
        Path::new(&home_path).join(".skoczek.json")
    };

    let app_m = cli::Cli::parse();
    let config_path = app_m
        .config
        .map(PathBuf::from)
        .unwrap_or(default_config_path);

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

    match app_m.command {
        cli::Commands::Command(args) => cmd::command_cmd(args, &mut config, &config_path)?,
        cli::Commands::Default(args) => cmd::command_default(args, &mut config, &config_path)?,
        cli::Commands::Get(args) => cmd::command_get(args, &config),
        cli::Commands::Ls(args) => cmd::command_ls(args, &config),
        cli::Commands::Mv(args) => cmd::command_mv(args, &mut config, &config_path)?,
        cli::Commands::Rm(args) => cmd::command_rm(args, &mut config, &config_path)?,
        cli::Commands::Set(args) => cmd::command_set(args, &mut config, &config_path)?,
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
