use clap::{App, AppSettings, Arg, SubCommand};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, path::Path};

const COMMAND_LS: &str = "ls";
const COMMAND_RM: &str = "rm";
const COMMAND_GET: &str = "get";
const COMMAND_SET: &str = "set";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub aliases: HashMap<String, String>,
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
                    .help("Path assigned to an alias")
                    .required(true)
                    .index(2),
            ),
        SubCommand::with_name(COMMAND_LS)
            .about("Displays known aliases and their paths")
            .alias("list")
            .arg(
                Arg::with_name("show_paths")
                    .short("p")
                    .long("show-paths")
                    .help("Display paths next to aliases"),
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
        (COMMAND_GET, Some(sub_m)) => {
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
        (COMMAND_SET, Some(sub_m)) => {
            let alias = sub_m
                .value_of("alias")
                .expect("alias is required")
                .to_owned();
            let path = sub_m
                .value_of("path")
                .expect("path is required arg")
                .to_owned();

            let res = config.aliases.insert(alias.clone(), path.clone());
            save_config_file(config_path, &config)?;
            // Print new alias only if database successfully saved
            match res {
                Some(prev_path) => {
                    println!("{}\t{} -> {}", alias, prev_path, path);
                }
                None => println!("{}\tnone -> {}", alias, path),
            }
        }
        (COMMAND_LS, Some(sub_m)) => {
            for (alias, path) in &config.aliases {
                if sub_m.is_present("show_paths") {
                    println!("{}\t{}", alias, path);
                } else {
                    println!("{}", alias);
                }
            }
        }
        (COMMAND_RM, Some(sub_m)) => {
            let alias = sub_m.value_of("alias").expect("alias is required");
            if config.aliases.remove(alias).is_none() {
                std::process::exit(1);
            }
            save_config_file(config_path, &config)?;
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn save_config_file(config_path: &str, config: &Config) -> anyhow::Result<()> {
    let config_file = File::create(config_path)?;
    let _ = serde_json::to_writer(config_file, config)?;
    Ok(())
}

fn main() {
    if run().is_err() {
        std::process::exit(1);
    }
}
