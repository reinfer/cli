#![deny(clippy::all)]
mod args;
mod commands;
mod config;
mod printer;
mod progress;
mod thousands;
mod utils;

use anyhow::{anyhow, Context, Result};
use commands::{
    auth::{self, refresh_user_permissions},
    package,
};
use log::{error, warn};
use once_cell::sync::Lazy;
use reinfer_client::{
    retry::{RetryConfig, RetryStrategy},
    Client, Config as ClientConfig, ProjectName, Token, DEFAULT_ENDPOINT,
};
use scoped_threadpool::Pool;
use std::{env, fs, io, path::PathBuf, process};
use structopt::{clap::Shell as ClapShell, StructOpt};

use crate::{
    args::{Args, Command, Shell},
    commands::{config as config_command, create, delete, get, parse, update},
    config::ReinferConfig,
    printer::Printer,
};

const NUM_THREADS_ENV_VARIABLE_NAME: &str = "REINFER_CLI_NUM_THREADS";

static DEFAULT_PROJECT_NAME: Lazy<ProjectName> =
    Lazy::new(|| ProjectName("DefaultProject".to_string()));

fn run(args: Args) -> Result<()> {
    let config_path = find_configuration(&args)?;
    let config = config::read_reinfer_config(&config_path)?;
    let printer = Printer::new(args.output);

    let number_of_threads = if let Ok(num_threads_env_var_str) =
        env::var(NUM_THREADS_ENV_VARIABLE_NAME)
    {
        num_threads_env_var_str
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("Environment variable {NUM_THREADS_ENV_VARIABLE_NAME} is not a u32: '{num_threads_env_var_str}'"))
    } else {
        args.num_threads
    };

    let mut pool = Pool::new(number_of_threads);

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
        Command::Get { get_args } => get::run(
            get_args,
            client_from_args(&args, &config)?,
            &printer,
            &mut pool,
        ),
        Command::Delete { delete_args } => {
            delete::run(delete_args, client_from_args(&args, &config)?)
        }
        Command::Create { create_args } => create::run(
            create_args,
            get_client_and_refresh_permission(&args, &config)?,
            &printer,
            &mut pool,
        ),
        Command::Update { update_args } => update::run(
            update_args,
            get_client_and_refresh_permission(&args, &config)?,
            &printer,
        ),
        Command::Parse { parse_args } => parse::run(
            parse_args,
            get_client_and_refresh_permission(&args, &config)?,
            &mut pool,
        ),

        Command::Package { package_args } => package::run(
            package_args,
            get_client_and_refresh_permission(&args, &config)?,
            &mut pool,
        ),
        Command::Authentication { auth_args } => auth::run(
            auth_args,
            get_client_and_refresh_permission(&args, &config)?,
        ),
    }
}

fn get_client_and_refresh_permission(args: &Args, config: &ReinferConfig) -> Result<Client> {
    let client = client_from_args(args, config)?;
    refresh_user_permissions(&client, false)?;
    Ok(client)
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

    let proxy = args
        .proxy
        .clone()
        .or_else(|| current_context.and_then(|context| context.proxy.clone()));

    // Retry everything but the very first request.
    // Retry wait schedule is [5s, 10s, 20s, fail]. (Plus the time for each attempt to timeout.)
    let retry_config = RetryConfig {
        strategy: RetryStrategy::Always,
        max_retry_count: 3,
        base_wait: std::time::Duration::from_secs_f64(5.0),
        backoff_factor: 2.0,
    };

    let client = Client::new(ClientConfig {
        endpoint,
        token,
        accept_invalid_certificates,
        proxy,
        retry_config: Some(retry_config),
    })
    .context("Failed to initialise the HTTP client.")?;

    check_if_context_is_a_required_field(config, &client, args)?;

    Ok(client)
}

const DOMAINS_THAT_REQUIRE_CONTEXT: [&str; 2] = ["uipath.com", "reinfer.dev"];

fn check_if_context_is_a_required_field(
    config: &ReinferConfig,
    client: &Client,
    args: &Args,
) -> Result<()> {
    let context_is_none = args.context.is_none() && args.endpoint.is_none();

    if config.context_is_required && context_is_none {
        return Err(anyhow!(
            "Please provide a context with the `re -c <context>` option or opt out with `re config set-context-required false`"
        ));
    }

    let current_user = client.get_current_user()?;

    if DOMAINS_THAT_REQUIRE_CONTEXT
        .iter()
        .any(|domain| current_user.email.0.to_lowercase().ends_with(domain))
        && context_is_none
    {
        return Err(anyhow!(
            "As a UiPath user, please provide a context with the `re -c <context>` option"
        ));
    };

    Ok(())
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
            error!(" |- {cause}");
        }

        #[cfg(feature = "backtrace")]
        {
            error!("{}", error.backtrace());
        }

        process::exit(1);
    }
}
