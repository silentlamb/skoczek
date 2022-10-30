use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    /// Path to configuration (default: ~/.skoczek.json)
    #[arg(short, long)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Command(CmdCommand),
    Default(CmdDefault),
    Get(CmdGet),

    #[command(aliases = &["list"])]
    Ls(CmdLs),
    Mv(CmdMv),
    Rm(CmdRm),
    Set(CmdSet),
}

// Get/set command(s) to be run after the jump
#[derive(Args)]
pub struct CmdCommand {
    /// Alias name
    pub alias: String,

    #[arg(short = 's', long = "set")]
    pub command: Option<String>,
}

/// Get/set default alias
#[derive(Args)]
pub struct CmdDefault {
    /// Sets given alias as a default one
    #[arg(short = 's', long = "set")]
    pub alias: Option<String>,
}

/// Displays path for a given alias
#[derive(Args)]
pub struct CmdGet {
    /// Alias for which to display a path
    pub alias: String,
}

/// Displays known aliases and their paths
#[derive(Args)]
pub struct CmdLs {
    /// Display paths next to aliases
    #[arg(short = 'p', long)]
    pub show_paths: bool,

    /// Display all paths
    #[arg(short, long, conflicts_with = "remote_only")]
    pub all: bool,

    /// Display remote paths only
    #[arg(short, long = "remote", conflicts_with = "all")]
    pub remote_only: bool,
}

/// Rename an alias
#[derive(Args)]
pub struct CmdMv {
    /// Alias to rename
    pub alias_from: String,

    /// Destination alias name
    pub alias_to: String,

    /// Rename if destination alias name already exists
    #[arg(short, long)]
    pub force: bool,
}

/// Removes an alias
#[derive(Args)]
pub struct CmdRm {
    /// Alias of a path to remove
    pub alias: String,
}

/// Assigns alias to a path
#[derive(Args)]
pub struct CmdSet {
    /// Alias of a path (default: last part of CMD)
    pub alias: Option<String>,

    /// Path assigned to an alais (default: CWD)
    pub path: Option<String>,

    /// Replace path if alias already exists
    #[arg(short, long)]
    pub force: bool,

    /// Set path to specific remote host
    #[arg(short, long)]
    pub remote: Option<String>,
}
