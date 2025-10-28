#![deny(clippy::all)]

// Module declarations
mod args;
mod commands;
mod config;
mod printer;
mod progress;
mod thousands;
mod utils;

// External crate imports
use anyhow::{anyhow, Context, Result};
use log::{error, warn};
use once_cell::sync::Lazy;
use openapi::apis::configuration::Configuration;
use reqwest::Url;
use scoped_threadpool::Pool;
use std::{env, fs, io, path::PathBuf, process};
use structopt::{clap::Shell as ClapShell, StructOpt};

// Internal crate imports
use crate::{
    args::{Args, Command, Shell},
    commands::{auth, config as config_command, create, delete, get, package, parse, update},
    config::ReinferConfig,
    printer::Printer,
    utils::{
        get_current_user,
        io::{init_env_logger, read_token_from_stdin},
        refresh_user_permissions, ProjectName,
    },
};

const NUM_THREADS_ENV_VARIABLE_NAME: &str = "REINFER_CLI_NUM_THREADS";

// OpenAPI equivalent constants
static DEFAULT_ENDPOINT: Lazy<Url> =
    Lazy::new(|| Url::parse("https://reinfer.dev").expect("Default URL is well-formed"));

// Simple wrapper for tokens (replaces reinfer_client::Token)
#[derive(Debug, Clone)]
pub struct Token(pub String);

static DEFAULT_PROJECT_NAME: Lazy<ProjectName> =
    Lazy::new(|| ProjectName("DefaultProject".to_string()));

fn run(args: Args) -> Result<()> {
    let config_path = find_configuration(&args)?;
    let cli_config = config::read_reinfer_config(&config_path)?;
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
            config_command::run(config_args, cli_config, config_path).map(|_| ())
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
            &config_from_args(&args, &cli_config)?,
            &printer,
            &mut pool,
        ),
        Command::Delete { delete_args } => {
            delete::run(delete_args, &config_from_args(&args, &cli_config)?)
        }
        Command::Create { create_args } => create::run(
            create_args,
            &get_config_and_refresh_permission(&args, &cli_config)?,
            &printer,
            &mut pool,
        ),
        Command::Update { update_args } => update::run(
            update_args,
            &get_config_and_refresh_permission(&args, &cli_config)?,
            &printer,
        ),
        Command::Parse { parse_args } => {
            let config = get_config_and_refresh_permission(&args, &cli_config)?;
            parse::run(parse_args, &config, &mut pool)
        }

        Command::Package { package_args } => {
            let config = get_config_and_refresh_permission(&args, &cli_config)?;
            package::run(package_args, &config, &mut pool)
        }
        Command::Authentication { auth_args } => auth::run(
            auth_args,
            &get_config_and_refresh_permission(&args, &cli_config)?,
        ),
    }
}

// Legacy functions removed - all commands now use OpenAPI Configuration

/// Create OpenAPI Configuration with EXACT same settings as client_from_args()
fn config_from_args(args: &Args, config: &ReinferConfig) -> Result<Configuration> {
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
        read_token_from_stdin()?.unwrap_or_default()
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

    // Create OpenAPI config with EXACT same TLS behavior as legacy client
    let mut builder = reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certificates) // ← SAME as legacy!
        .timeout(std::time::Duration::from_secs(240));

    // Handle proxy (same as legacy client)
    let proxy = args
        .proxy
        .clone()
        .or_else(|| current_context.and_then(|context| context.proxy.clone()));

    if let Some(proxy) = proxy {
        builder = builder.proxy(reqwest::Proxy::all(proxy).context("Failed to configure proxy")?);
    }

    let client = builder
        .build()
        .context("Failed to initialise the HTTP client.")?;

    // Create OpenAPI configuration with all fields initialized
    let openapi_config = Configuration {
        base_path: endpoint.to_string(),
        api_key: Some(openapi::apis::configuration::ApiKey {
            prefix: Some("Bearer".to_string()),
            key: token.0.clone(),
        }),
        bearer_access_token: Some(token.0),
        client, // ← Use our TLS-configured client
        ..Default::default()
    };

    // Check context requirements (mirrors legacy client behavior)
    check_if_context_is_a_required_field(config, &openapi_config, args)?;

    Ok(openapi_config)
}

/// Create OpenAPI Configuration and refresh permissions (mirrors get_client_and_refresh_permission)
fn get_config_and_refresh_permission(args: &Args, config: &ReinferConfig) -> Result<Configuration> {
    let openapi_config = config_from_args(args, config)?;

    // Refresh permissions using the same Configuration's TLS settings
    refresh_user_permissions(&openapi_config)?;

    Ok(openapi_config)
}

const DOMAINS_THAT_REQUIRE_CONTEXT: [&str; 2] = ["uipath.com", "reinfer.dev"];

fn check_if_context_is_a_required_field(
    config: &ReinferConfig,
    openapi_config: &Configuration,
    args: &Args,
) -> Result<()> {
    let context_is_none = args.context.is_none() && args.endpoint.is_none();

    if config.context_is_required && context_is_none {
        return Err(anyhow!(
            "Please provide a context with the `re -c <context>` option or opt out with `re config set-context-required false`"
        ));
    }

    let current_user = get_current_user(openapi_config)?;

    if DOMAINS_THAT_REQUIRE_CONTEXT
        .iter()
        .any(|domain| current_user.email.to_lowercase().ends_with(domain))
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
    init_env_logger(args.verbose);

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
