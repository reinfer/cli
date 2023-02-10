#![deny(clippy::all)]
mod args;
mod commands;
mod config;
mod printer;
mod progress;
mod utils;

use anyhow::{anyhow, Context, Result};
use log::{error, warn};
use reinfer_client::{
    retry::{RetryConfig, RetryStrategy},
    Client, Config as ClientConfig, Token, DEFAULT_ENDPOINT,
};
use std::{fs, io, path::PathBuf, process};
use structopt::{clap::Shell as ClapShell, StructOpt};

use crate::{
    args::{Args, Command, Shell},
    commands::{config as config_command, create, delete, get, update},
    config::ReinferConfig,
    printer::Printer,
};

fn run(args: Args) -> Result<()> {
    let config_path = find_configuration(&args)?;
    let config = config::read_reinfer_config(&config_path)?;
    let printer = Printer::new(args.output);

    match &args.command {
        Command::Config { config_args } => {
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
        Command::Get { get_args } => {
            get::run(get_args, client_from_args(&args, &config)?, &printer)
        }
        Command::Delete { delete_args } => {
            delete::run(delete_args, client_from_args(&args, &config)?)
        }
        Command::Create { create_args } => {
            create::run(create_args, client_from_args(&args, &config)?, &printer)
        }
        Command::Update { update_args } => {
            update::run(update_args, client_from_args(&args, &config)?, &printer)
        }
    }
}

fn client_from_args(args: &Args, config: &ReinferConfig) -> Result<Client> {
    let current_context = if let Some(context_name) = args.context.as_ref() {
        let context = config.get_context(context_name);
        if context.is_none() {
            return Err(anyhow!("Unknown context `{}`.", context_name));
        };
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
        utils::read_token_from_stdin()?.unwrap_or_default()
    });

    let accept_invalid_certificates = args.accept_invalid_certificates
        || current_context
            .map(|context| context.accept_invalid_certificates)
            .unwrap_or(false);

    if accept_invalid_certificates {
        warn!(concat!(
            "TLS certificate verification is disabled. ",
            "Do NOT use this over an insecure network."
        ));
    }

    let proxy = args
        .proxy
        .clone()
        .or_else(|| current_context.and_then(|context| context.proxy.clone()));

    // Retry everything but the very first request.
    // Retry wait schedule is [5s, 10s, 20s, fail]. (Plus the time for each attempt to timeout.)
    let retry_config = RetryConfig {
        strategy: RetryStrategy::Automatic,
        max_retry_count: 3,
        base_wait: std::time::Duration::from_secs_f64(5.0),
        backoff_factor: 2.0,
    };

    Client::new(ClientConfig {
        endpoint,
        token,
        accept_invalid_certificates,
        proxy,
        retry_config: Some(retry_config),
    })
    .context("Failed to initialise the HTTP client.")
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
        let mut config_path =
            dirs::config_dir().context("Could not get path to the user's config directory")?;
        config_path.push("reinfer");
        fs::create_dir_all(&config_path).with_context(|| {
            format!(
                "Could not create config directory {}",
                config_path.display()
            )
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
        error!("An error occurred:");
        for cause in error.chain() {
            error!(" |- {}", cause);
        }

        #[cfg(feature = "backtrace")]
        {
            error!("{}", error.backtrace());
        }

        process::exit(1);
    }
}
