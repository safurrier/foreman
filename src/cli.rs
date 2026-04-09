use crate::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use crate::app::{AppState, InventorySummary};
use crate::config::{load_config, resolve_paths, write_default_config, ConfigError, RuntimeConfig};
use crate::services::logging::{RunLogSummary, RunLogger};
use clap::Parser;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
#[command(name = "foreman", about = "TUI for managing AI agents across tmux")]
pub struct Cli {
    #[arg(long, conflicts_with = "init_config")]
    pub config_path: bool,

    #[arg(long, conflicts_with = "config_path")]
    pub init_config: bool,

    #[arg(long)]
    pub config_file: Option<PathBuf>,

    #[arg(long)]
    pub poll_interval_ms: Option<u64>,

    #[arg(long)]
    pub capture_lines: Option<usize>,

    #[arg(long)]
    pub popup: bool,

    #[arg(long = "no-notify")]
    pub no_notify: bool,

    #[arg(long, hide = true)]
    pub log_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub tmux_socket: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    PrintedConfigPath(PathBuf),
    InitializedConfig(PathBuf),
    Bootstrapped(Box<BootstrapSummary>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSummary {
    pub runtime: RuntimeConfig,
    pub logs: RunLogSummary,
    pub state: AppState,
    pub inventory: InventorySummary,
}

#[derive(Debug)]
pub enum RunError {
    Config(ConfigError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RunError {}

impl From<ConfigError> for RunError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

pub fn run(cli: Cli) -> Result<RunOutcome, RunError> {
    let paths = resolve_paths(cli.config_file.as_deref(), cli.log_dir.as_deref())?;

    if cli.config_path {
        println!("{}", paths.config_file.display());
        return Ok(RunOutcome::PrintedConfigPath(paths.config_file));
    }

    if cli.init_config {
        write_default_config(&paths.config_file)?;
        println!(
            "Initialized default config at {}",
            paths.config_file.display()
        );
        return Ok(RunOutcome::InitializedConfig(paths.config_file));
    }

    let file_config = load_config(&paths.config_file)?;
    let runtime = RuntimeConfig::from_sources(paths, file_config, &cli);

    let mut logger =
        RunLogger::start(&runtime.log_dir, runtime.log_retention).map_err(ConfigError::Io)?;
    logger.log_bootstrap(&runtime).map_err(ConfigError::Io)?;
    let state = bootstrap_state(&runtime);
    let inventory = state.inventory_summary();
    logger.log_inventory(&inventory).map_err(ConfigError::Io)?;
    if let Some(error) = &state.startup_error {
        logger.log_tmux_error(error).map_err(ConfigError::Io)?;
    }

    println!("Foreman bootstrap complete.");
    Ok(RunOutcome::Bootstrapped(Box::new(BootstrapSummary {
        runtime,
        logs: logger.summary(),
        state,
        inventory,
    })))
}

fn bootstrap_state(runtime: &RuntimeConfig) -> AppState {
    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(runtime.tmux_socket.clone()));
    match adapter.load_inventory(runtime.capture_lines) {
        Ok(inventory) => AppState::with_inventory(inventory),
        Err(error) => AppState {
            startup_error: Some(error.to_string()),
            ..AppState::default()
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{run, Cli, RunOutcome};
    use clap::Parser;
    use tempfile::tempdir;

    #[test]
    fn run_prints_resolved_config_path() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let cli = Cli::parse_from([
            "foreman",
            "--config-path",
            "--config-file",
            temp_dir
                .path()
                .join("custom.toml")
                .to_str()
                .expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("run should succeed");

        assert!(matches!(outcome, RunOutcome::PrintedConfigPath(_)));
    }

    #[test]
    fn run_bootstraps_when_no_utility_flag_is_provided() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let cli = Cli::parse_from([
            "foreman",
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("run should succeed");

        match outcome {
            RunOutcome::Bootstrapped(summary) => {
                assert!(summary.logs.run_path.exists());
                assert!(summary.logs.latest_path.exists());
            }
            other => panic!("expected bootstrapped outcome, got {other:?}"),
        }
    }
}
