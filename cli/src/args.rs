use crate::commands::{config::ConfigArgs, create::CreateArgs, delete::DeleteArgs, get::GetArgs};
use anyhow::{anyhow, Error, Result};
use reqwest::Url;
use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

/// re is the command line interface to reinfer clusters.
#[derive(Debug, StructOpt)]
#[structopt(
    global_settings = &[
        structopt::clap::AppSettings::ColoredHelp,
        structopt::clap::AppSettings::InferSubcommands,
    ]
)]
pub struct Args {
    #[structopt(long = "config-file", parse(from_os_str))]
    /// Path to the configuration file. Typically defaults to ~/.config/reinfer on Linux.
    pub config: Option<PathBuf>,

    #[structopt(short = "c", long = "context")]
    /// Specify what context to use. Overrides the current context, if any.
    pub context: Option<String>,

    #[structopt(short = "v", long = "verbose")]
    /// Enable more verbose logging.
    pub verbose: bool,

    #[structopt(long = "endpoint", parse(try_from_str))]
    /// Specify what endpoint to use. Overrides the one from the current
    /// context, if any.
    pub endpoint: Option<Url>,

    #[structopt(short = "k", long = "accept-invalid-certificates", parse(try_from_str))]
    pub accept_invalid_certificates: Option<bool>,

    #[structopt(long = "token")]
    /// Specify what API token to use. Overrides the one from the current
    /// context, if any.
    pub token: Option<String>,

    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "completion")]
    /// Output shell completion code for the specified shell (bash or zsh)
    Completion { shell: Shell },

    #[structopt(name = "config")]
    /// Manage reinfer authentication and endpoint contexts
    Config {
        #[structopt(subcommand)]
        config_args: ConfigArgs,
    },

    #[structopt(name = "create")]
    /// Create new resources
    Create {
        #[structopt(subcommand)]
        create_args: CreateArgs,
    },

    #[structopt(name = "delete")]
    /// Delete a resource
    Delete {
        #[structopt(subcommand)]
        delete_args: DeleteArgs,
    },

    #[structopt(name = "get")]
    /// Display resources and export comments to the local filesystem.
    Get {
        #[structopt(subcommand)]
        get_args: GetArgs,
    },
}

#[derive(Debug)]
pub enum Shell {
    Bash,
    Zsh,
}

impl FromStr for Shell {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self> {
        match string {
            "bash" => Ok(Shell::Bash),
            "zsh" => Ok(Shell::Zsh),
            _ => Err(anyhow!("unknown shell: '{}'", string)),
        }
    }
}
