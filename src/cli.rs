use clap::{App, AppSettings, Arg, SubCommand};

pub const COMMAND_LS: &str = "ls";
pub const COMMAND_RM: &str = "rm";
pub const COMMAND_GET: &str = "get";
pub const COMMAND_SET: &str = "set";
pub const COMMAND_MV: &str = "mv";
pub const COMMAND_DEFAULT: &str = "default";
pub const COMMAND_COMPLETIONS: &str = "completions";

pub fn build_cli() -> App<'static, 'static> {
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
        SubCommand::with_name(COMMAND_COMPLETIONS)
            .about("Generate shell completions")
            .arg(
                Arg::with_name("shell")
                    .help("Name of shell (bash, fish)")
                    .required(true)
                    .validator(is_supported_shell_name)
                    .index(1),
            ),
    ];

    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .value_name("FILE"),
        )
        .subcommands(commands)
}

fn is_supported_shell_name(v: String) -> Result<(), String> {
    let allowed: Vec<&str> = vec!["bash", "fish"];
    if allowed.contains(&v.as_str()) {
        return Ok(());
    }
    Err(String::from("supported shells: bash, fish"))
}
