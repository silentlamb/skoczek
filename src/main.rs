use clap::{App, AppSettings, Arg, SubCommand};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

const COMMAND_LS: &str = "ls";
const COMMAND_RM: &str = "rm";
const COMMAND_GET: &str = "get";
const COMMAND_SET: &str = "set";
const COMMAND_MV: &str = "mv";
const COMMAND_DEFAULT: &str = "default";

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, Destination>,

    #[serde(default)]
    pub default: Option<String>,
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

    let commands = vec![
        SubCommand::with_name(COMMAND_SET)
            .about("Assigns alias to a path")
            .arg(
                Arg::with_name("alias")
                    .help("Alias of a path")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("path")
                    .help("Path assigned to an alias (default: CWD)")
                    .index(2),
            )
            .arg(
                Arg::with_name("force")
                    .help("Replace path if alias already exists")
                    .short("f")
                    .long("force"),
            )
            .arg(
                Arg::with_name("remote")
                    .short("r")
                    .long("remote")
                    .help("Set path to specific remote host")
                    .takes_value(true),
            ),
        SubCommand::with_name(COMMAND_LS)
            .about("Displays known aliases and their paths")
            .alias("list")
            .arg(
                Arg::with_name("show_paths")
                    .short("p")
                    .long("show-paths")
                    .help("Display paths next to aliases"),
            )
            .arg(
                Arg::with_name("all")
                    .short("a")
                    .long("--all")
                    .help("Display all paths")
                    .conflicts_with_all(&["remote_only"]),
            )
            .arg(
                Arg::with_name("remote_only")
                    .short("r")
                    .long("--remote")
                    .help("Display only remote paths")
                    .conflicts_with_all(&["local_only"]),
            ),
        SubCommand::with_name(COMMAND_RM)
            .about("Removes an alias")
            .arg(
                Arg::with_name("alias")
                    .help("Alias of a path to remove")
                    .index(1)
                    .required(true),
            ),
        SubCommand::with_name(COMMAND_GET)
            .about("Displays path for a given alias")
            .arg(
                Arg::with_name("alias")
                    .help("Alias for which to display a path")
                    .required(true)
                    .index(1),
            ),
        SubCommand::with_name(COMMAND_MV)
            .about("Rename an alias")
            .arg(
                Arg::with_name("alias_from")
                    .help("Alias to rename")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("alias_to")
                    .help("Destination alias name")
                    .required(true)
                    .index(2),
            )
            .arg(
                Arg::with_name("force")
                    .help("Rename if destination alias name already exists")
                    .short("f")
                    .long("force"),
            ),
        SubCommand::with_name(COMMAND_DEFAULT)
            .about("Get/set default alias")
            .arg(
                Arg::with_name("set")
                    .help("Sets alias as default one")
                    .short("s")
                    .long("set")
                    .takes_value(true)
                    .value_name("ALIAS"),
            ),
    ];

    let app_m = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE")
                .default_value(&default_config_path),
        )
        .subcommands(commands)
        .get_matches();

    let config_path = app_m
        .value_of("config")
        .expect("default value should work?");
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
        _ => unreachable!(),
    }

    Ok(())
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

fn command_set(
    sub_m: &clap::ArgMatches,
    config: &mut Config,
    config_path: &str,
) -> Result<(), anyhow::Error> {
    let alias = sub_m
        .value_of("alias")
        .expect("alias is required")
        .to_owned();

    let path = match sub_m.value_of("remote") {
        Some(_) => match sub_m.value_of("path") {
            Some(path) => path.to_owned(),
            None => {
                eprintln!("CWD cannot be used for --remote");
                std::process::exit(1);
            }
        },
        None => sub_m
            .value_of("path")
            .map(|p| p.to_owned())
            .unwrap_or_else(
                || match std::env::current_dir().map(|p| p.into_os_string()) {
                    Ok(cwd) => cwd.into_string().unwrap(),
                    Err(err) => {
                        match err.kind() {
                            std::io::ErrorKind::NotFound => {
                                eprintln!("CWD cannot be used: path not found");
                            }
                            std::io::ErrorKind::PermissionDenied => {
                                eprintln!("CWD cannot be used: permission denied");
                            }
                            _ => {}
                        }
                        std::process::exit(1);
                    }
                },
            ),
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
            save_config_file(config_path, &config)?;
            println!("{} -> {}\t{}", alias_from, alias_to, path);
        }
        None => {
            eprintln!("Alias not found: {}", alias_from);
            std::process::exit(1);
        }
    }
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
                Some(path) => {
                    println!("{}", path);
                }
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

fn save_config_file(config_path: &str, config: &Config) -> anyhow::Result<()> {
    let config_file = File::create(config_path)?;
    let _ = serde_json::to_writer(config_file, config)?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
