use crate::{
    commands::{
        auth::AuthArgs, config::ConfigArgs, create::CreateArgs, delete::DeleteArgs, get::GetArgs,
        package::PackageArgs, parse::ParseArgs, update::UpdateArgs,
    },
    printer::OutputFormat,
};
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

    #[structopt(long = "proxy")]
    /// URL for an HTTP proxy that will be used for all requests if specified
    pub proxy: Option<Url>,

    #[structopt(short = "o", long = "output", default_value = "table")]
    /// Output format. One of: json, table
    ///
    /// Output is provided in table format on stdout by default.
    pub output: OutputFormat,

    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(long = "num-threads", default_value = "32")]
    /// The number of threads to use when uploading annotations and emls. Can be overwritten by the
    /// REINFER_CLI_NUM_THREADS environment variable
    pub num_threads: u32,
}

#[allow(clippy::large_enum_variant)]
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

    #[structopt(name = "update")]
    /// Update existing resources
    Update {
        #[structopt(subcommand)]
        update_args: UpdateArgs,
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

    #[structopt(name = "parse")]
    /// Upload data from various file types
    Parse {
        #[structopt(subcommand)]
        parse_args: ParseArgs,
    },

    #[structopt(name = "package")]
    /// Create packages for moving data around
    Package {
        #[structopt(subcommand)]
        package_args: PackageArgs,
    },

    #[structopt(name = "auth")]
    /// Manage authentication for the current user
    Authentication {
        #[structopt(subcommand)]
        auth_args: AuthArgs,
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
