use anyhow::{Context, Result};
use colored::{ColoredString, Colorize};
use env_logger::{fmt::Formatter as LogFormatter, Builder as LogBuilder};
use lazy_static::lazy_static;
use log::{Level as LogLevel, LevelFilter as LogLevelFilter, Record as LogRecord};
use std::{
    env,
    io::{self, Write},
    ops::Deref,
};

pub fn init_env_logger(verbose: bool) {
    let format = |formatter: &mut LogFormatter, record: &LogRecord<'_>| {
        let level = match record.level() {
            LogLevel::Debug => LOG_PREFIX_DEBUG.deref(),
            LogLevel::Info => LOG_PREFIX_INFO.deref(),
            LogLevel::Warn => LOG_PREFIX_WARN.deref(),
            LogLevel::Error => LOG_PREFIX_ERROR.deref(),
            LogLevel::Trace => LOG_PREFIX_TRACE.deref(),
        };
        writeln!(formatter, "{} {}", level, record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format).filter(
        None,
        if verbose {
            LogLevelFilter::Debug
        } else {
            LogLevelFilter::Info
        },
    );

    if env::var("RUST_LOG").is_ok() {
        builder.parse_filters(&env::var("RUST_LOG").unwrap());
    }

    builder.init();
}

pub fn read_from_stdin(message: &str, default: Option<&str>) -> Result<String> {
    let mut input = String::new();
    write!(
        io::stderr(),
        "{} {}{}: ",
        LOG_PREFIX_INPUT.deref(),
        message,
        if let Some(value) = default {
            format!(" [{}]", value)
        } else {
            "".into()
        },
    )
    .and_then(|_| io::stderr().flush())
    .and_then(|_| io::stdin().read_line(&mut input))
    .context("Failed to read from stdin.")?;
    input = input.trim().into();
    Ok(match (input.is_empty(), default) {
        (true, Some(default)) => default.into(),
        _ => input,
    })
}

pub fn read_token_from_stdin() -> Result<Option<String>> {
    let mut input = String::new();
    write!(
        io::stderr(),
        "{} Enter API token [none]: ",
        LOG_PREFIX_INPUT.deref()
    )
    .and_then(|_| io::stderr().flush())
    .and_then(|_| io::stdin().read_line(&mut input))
    .context("Failed to read API token from stdin.")?;
    input = input.trim().into();
    Ok(if !input.is_empty() { Some(input) } else { None })
}

lazy_static! {
    pub static ref LOG_PREFIX_DEBUG: ColoredString = "D".normal();
    pub static ref LOG_PREFIX_INFO: ColoredString = "I".green();
    pub static ref LOG_PREFIX_WARN: ColoredString = "W".yellow().bold();
    pub static ref LOG_PREFIX_ERROR: ColoredString = "E".red().bold();
    pub static ref LOG_PREFIX_TRACE: ColoredString = "T".normal();
    pub static ref LOG_PREFIX_INPUT: ColoredString = "*".blue().bold();
}
