#![deny(clippy::all)]
mod args;
mod commands;
mod config;
mod errors;
mod progress;
mod utils;

use failchain::{ensure, ResultExt};
use failure::{AsFail, Fail};
use log::{error, warn};
use reinfer_client::{Client, Config as ClientConfig, Token, DEFAULT_ENDPOINT};
use std::{env, fs, io, path::PathBuf, process};
use structopt::{clap::Shell as ClapShell, StructOpt};

use crate::{
    args::{Args, Command, Shell},
    commands::{config as config_command, create, delete, get, task},
    config::ReinferConfig,
    errors::{Error, ErrorKind, Result},
};

fn run(args: Args) -> Result<()> {
    let config_path = find_configuration(&args)?;
    let config = config::read_reinfer_config(&config_path)?;

    match args.command {
        Command::Config { ref config_args } => {
            config_command::run(config_args, config, config_path).map(|_| ())
        }
        Command::Completion { shell } => {
            let mut app = Args::clap();
            let clap_shell = match shell {
                Shell::Zsh => ClapShell::Zsh,
                Shell::Bash => ClapShell::Bash,
            };
            app.gen_completions_to("re", clap_shell, &mut io::stdout());
            Ok(())
        }
        Command::Get { ref get_args } => get::run(get_args, client_from_args(&args, &config)?),
        Command::Delete { ref delete_args } => {
            delete::run(delete_args, client_from_args(&args, &config)?)
        }
        Command::Create { ref create_args } => {
            create::run(create_args, client_from_args(&args, &config)?)
        }
        Command::Task { ref task_args } => task::run(
            task_args,
            client_from_args(&args, &config)?,
            args.experimental,
        ),
    }
}

fn client_from_args(args: &Args, config: &ReinferConfig) -> Result<Client> {
    let current_context = if let Some(context_name) = args.context.as_ref() {
        let context = config.get_context(context_name);
        ensure!(
            context.is_some(),
            ErrorKind::Config,
            "Unknown context `{}`.",
            context_name
        );
        context
    } else {
        config.get_current_context()
    };

    let endpoint = args
        .endpoint
        .clone()
        .or_else(|| current_context.map(|context| context.endpoint.clone()))
        .unwrap_or_else(|| DEFAULT_ENDPOINT.clone());

    let args_or_config_token = args
        .token
        .clone()
        .or_else(|| current_context.and_then(|context| context.token.clone()));

    let token = Token(if let Some(token) = args_or_config_token {
        token
    } else {
        utils::read_token_from_stdin()?.unwrap_or_else(String::new)
    });

    let accept_invalid_certificates = args
        .accept_invalid_certificates
        .or_else(|| current_context.map(|context| context.accept_invalid_certificates))
        .unwrap_or(false);

    if accept_invalid_certificates {
        warn!(concat!(
            "TLS certificate verification is disabled. ",
            "Do NOT use this over an insecure network."
        ));
    }

    Client::new(ClientConfig {
        endpoint,
        token,
        accept_invalid_certificates,
    })
    .chain_err(|| ErrorKind::Client("Failed to initialise the HTTP client.".into()))
}

fn find_configuration(args: &Args) -> Result<PathBuf> {
    let config_path = if let Some(config_path) = args.config.clone() {
        if !config_path.exists() {
            warn!(
                "Configuration file `{}` doesn't exist.",
                config_path.display()
            );
        }
        config_path
    } else {
        let mut config_path = dirs::config_dir().ok_or_else::<Error, _>(|| {
            ErrorKind::Config("Could not get path to the user's config directory".into()).into()
        })?;
        config_path.push("reinfer");
        fs::create_dir_all(&config_path).chain_err(|| {
            ErrorKind::Config(format!(
                "Could not create config directory {}",
                config_path.display()
            ))
        })?;
        config_path.push("contexts.json");
        config_path
    };
    Ok(config_path)
}

fn main() {
    let args = Args::from_args();
    utils::init_env_logger(args.verbose);

    if let Err(error) = run(args) {
        error!("{}", error);
        let mut cause = error.as_fail();
        while let Some(new_cause) = cause.cause() {
            cause = new_cause;
            error!(" |- {}", cause);
        }
        if env::var("RUST_BACKTRACE")
            .map(|value| value == "1")
            .unwrap_or(false)
        {
            if let Some(backtrace) = error.backtrace() {
                error!("{:?}", backtrace);
            }
        }
        process::exit(1);
    }
}
