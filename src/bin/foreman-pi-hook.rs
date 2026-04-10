use clap::{Parser, ValueEnum};
use foreman::config::{default_pi_native_dir, load_config, resolve_paths, ConfigError};
use foreman::integrations::{
    bridge_pi_event, PiHookBridgeError, PiHookBridgeRequest, PiHookEventKind,
};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "foreman-pi-hook",
    about = "Bridge Pi extension lifecycle events into Foreman's native signal files"
)]
struct Cli {
    #[arg(long)]
    native_dir: Option<PathBuf>,

    #[arg(long)]
    config_file: Option<PathBuf>,

    #[arg(long, hide = true)]
    log_dir: Option<PathBuf>,

    #[arg(long)]
    pane_id: Option<String>,

    #[arg(long, value_enum)]
    event: EventArg,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum EventArg {
    BeforeAgentStart,
    AgentStart,
    AgentEnd,
    SessionShutdown,
}

impl From<EventArg> for PiHookEventKind {
    fn from(value: EventArg) -> Self {
        match value {
            EventArg::BeforeAgentStart => Self::BeforeAgentStart,
            EventArg::AgentStart => Self::AgentStart,
            EventArg::AgentEnd => Self::AgentEnd,
            EventArg::SessionShutdown => Self::SessionShutdown,
        }
    }
}

#[derive(Debug)]
enum HookCliError {
    Config(ConfigError),
    Bridge(PiHookBridgeError),
}

impl fmt::Display for HookCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::Bridge(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for HookCliError {}

impl From<ConfigError> for HookCliError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<PiHookBridgeError> for HookCliError {
    fn from(error: PiHookBridgeError) -> Self {
        Self::Bridge(error)
    }
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), HookCliError> {
    let native_dir = resolve_native_dir(&cli)?;
    let request = PiHookBridgeRequest::with_tmux_pane(native_dir, cli.pane_id)?;
    bridge_pi_event(&request, cli.event.into())?;
    Ok(())
}

fn resolve_native_dir(cli: &Cli) -> Result<PathBuf, HookCliError> {
    if let Some(path) = &cli.native_dir {
        return Ok(path.clone());
    }

    let paths = resolve_paths(cli.config_file.as_deref(), cli.log_dir.as_deref())?;
    let file_config = load_config(&paths.config_file)?;
    Ok(file_config
        .integrations
        .pi
        .native_dir
        .unwrap_or_else(|| default_pi_native_dir(&paths.log_dir)))
}
