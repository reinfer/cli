use colored::Colorize;
use log::{error, info, warn};
use prettytable::{self, cell, row, Table};
use reinfer_client::DEFAULT_ENDPOINT;
use reqwest::Url;
use std::path::Path;
use structopt::StructOpt;

use crate::{
    config::{self, ContextConfig, ReinferConfig},
    utils,
};
use anyhow::Result;

#[derive(Debug, StructOpt)]
pub enum ConfigArgs {
    #[structopt(name = "add")]
    /// Add a new context to the reinfer config file
    AddContext {
        #[structopt(long = "name", short = "n")]
        /// The name of the context that will be created or updated
        name: Option<String>,

        #[structopt(long = "endpoint", short = "e")]
        /// The reinfer cluster endpoint that will be used for this context
        endpoint: Option<Url>,

        #[structopt(long = "token", short = "t")]
        /// The reinfer API token that will be used for this context
        token: Option<String>,

        #[structopt(long = "accept-invalid-certificates", short = "k")]
        /// Whether to accept invalid TLS certificates
        accept_invalid_certificates: bool,

        #[structopt(long = "proxy")]
        /// URL for an HTTP proxy that will be used for all requests if specified
        proxy: Option<Option<Url>>,
    },

    #[structopt(name = "current")]
    /// Display the current context
    CurrentContext,

    #[structopt(name = "delete")]
    /// Delete the specified context from the reinfer config file
    DeleteContext {
        /// The name(s) of the context(s) which will be deleted
        names: Vec<String>,
    },

    #[structopt(name = "ls")]
    /// List available contexts in a reinfer config file
    ListContexts {
        #[structopt(long = "tokens")]
        /// Show API tokens (by default tokens are hidden).
        tokens: bool,
    },

    #[structopt(name = "use")]
    /// Set the current context in the reinfer config file
    UseContext {
        /// The name of the context.
        name: String,
    },
}

pub fn run(
    args: &ConfigArgs,
    mut config: ReinferConfig,
    config_path: impl AsRef<Path>,
) -> Result<ReinferConfig> {
    match args {
        ConfigArgs::ListContexts { tokens } if config.num_contexts() > 0 => {
            let mut contexts = config.get_all_contexts().clone();
            contexts.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));
            let mut table = new_table();
            table.set_titles(
                row![bFg => "Active", "Context", "Endpoint", "Insecure", "Token", "Proxy"],
            );
            for context in contexts.iter() {
                let active = config
                    .get_current_context()
                    .map_or(false, |current_context| {
                        current_context.name == context.name
                    });
                table.add_row(row![
                    if active { "    ->" } else { "" },
                    if active {
                        context.name.bold().bright_white()
                    } else {
                        context.name.normal()
                    },
                    context.endpoint,
                    if context.accept_invalid_certificates {
                        "Yes"
                    } else {
                        "No"
                    },
                    if *tokens {
                        context.token.clone().unwrap_or_else(String::new)
                    } else {
                        "<Hidden>".into()
                    },
                    context
                        .proxy
                        .clone()
                        .map(|url| url.to_string())
                        .unwrap_or_else(String::new)
                ]);
            }
            table.printstd();
        }
        ConfigArgs::ListContexts { .. } => {
            info!("No available contexts.");
        }
        ConfigArgs::AddContext {
            name,
            endpoint,
            token,
            accept_invalid_certificates,
            proxy,
        } => {
            add_or_edit_context(
                name,
                token,
                endpoint,
                *accept_invalid_certificates,
                proxy,
                config.clone(),
                config_path,
            )?;
        }
        ConfigArgs::UseContext { name } => {
            if !config.set_current_context(&name) {
                error!(
                    "No such context `{}` exists in `{}`.",
                    name,
                    config_path.as_ref().display()
                );
            } else {
                config::write_reinfer_config(config_path, &config)?;
                info!("Switched to context `{}`.", name);
            }
        }
        ConfigArgs::CurrentContext => config.get_current_context().map_or_else(
            || info!("There is no default context in use."),
            |current_context| println!("{}", current_context.name),
        ),
        ConfigArgs::DeleteContext { names } => {
            for name in names {
                if config.delete_context(name) {
                    config::write_reinfer_config(&config_path, &config)?;
                    info!(
                        "Deleted context `{}` from `{}`.",
                        name,
                        config_path.as_ref().display()
                    );
                } else {
                    error!(
                        "No such context `{}` exists in `{}`.",
                        name,
                        config_path.as_ref().display()
                    );
                }
            }
        }
    }
    Ok(config)
}

fn add_or_edit_context(
    name: &Option<String>,
    token: &Option<String>,
    endpoint: &Option<Url>,
    accept_invalid_certificates: bool,
    proxy: &Option<Option<Url>>,
    mut config: ReinferConfig,
    config_path: impl AsRef<Path>,
) -> Result<()> {
    // Get context name (either argument or from stdin)
    let name = loop {
        let name = match name {
            None => utils::read_from_stdin("Context name", None)?,
            Some(name) => name.clone(),
        };
        if !name.is_empty() {
            break name;
        } else {
            error!("Context name cannot be empty.");
        }
    };

    let existing_context = config.get_context(&name);
    if existing_context.is_some() {
        info!("Context `{}` already exists, it will be modified.", name);
    } else {
        info!("A new context `{}` will be created.", name);
    }

    // Get API token (either argument or from stdin)
    let token = match token {
        None => utils::read_token_from_stdin()?,
        token => token.clone(),
    };
    if token.is_none() {
        info!(concat!(
            "No API token was associated with the context. ",
            "You will have to enter it for every request."
        ));
    } else {
        warn!(
            "Be careful, API tokens are stored in cleartext in {}.",
            config_path.as_ref().display()
        );
    }

    // Get endpoint (either argument or from stdin)
    let endpoint = match endpoint {
        None => loop {
            match Url::parse(&utils::read_from_stdin(
                "Endpoint",
                Some(
                    existing_context
                        .as_ref()
                        .map(|context| context.endpoint.as_str())
                        .unwrap_or_else(|| DEFAULT_ENDPOINT.as_str()),
                ),
            )?) {
                Ok(url) => break url,
                Err(error) => {
                    error!("Invalid endpoint URL: {}", error);
                }
            }
        },
        Some(endpoint) => endpoint.clone(),
    };

    // Update the contexts' JSON configuration file
    let context = ContextConfig {
        name: name.clone(),
        endpoint,
        token,
        accept_invalid_certificates,
        proxy: proxy.clone().unwrap_or_else(|| {
            existing_context
                .as_ref()
                .and_then(|context| context.proxy.clone())
        }),
    };

    let update_existing = existing_context.is_some();
    let is_new_context = !config.set_context(context);
    if is_new_context && config.num_contexts() == 1 {
        info!("Default context set to `{}`.", name);
        config.set_current_context(&name);
    }

    config::write_reinfer_config(config_path, &config)?;

    if update_existing {
        info!("Context `{}` was updated.", name);
    } else {
        info!("New context `{}` was created.", name);
    }

    Ok(())
}

fn new_table() -> Table {
    let mut table = Table::new();
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .separators(
            &[],
            prettytable::format::LineSeparator::new('-', '+', '+', '+'),
        )
        .padding(0, 1)
        .build();
    table.set_format(format);
    table
}
