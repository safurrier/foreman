use crate::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use crate::app::{
    AppState, InventorySummary, OperatorAlert, OperatorAlertLevel, OperatorAlertSource,
};
use crate::config::{load_config, resolve_paths, write_default_config, ConfigError, RuntimeConfig};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::logging::{RunLogSummary, RunLogger};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
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

    #[arg(long)]
    pub debug: bool,

    #[arg(long, hide = true)]
    pub log_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub tmux_socket: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub claude_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub codex_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub pi_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub bootstrap_only: bool,
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
    pub claude_native: ClaudeNativeOverlaySummary,
    pub codex_native: CodexNativeOverlaySummary,
    pub pi_native: PiNativeOverlaySummary,
}

pub(crate) struct PreparedBootstrap {
    pub runtime: RuntimeConfig,
    pub logger: RunLogger,
    pub state: AppState,
    pub claude_native: ClaudeNativeOverlaySummary,
    pub codex_native: CodexNativeOverlaySummary,
    pub pi_native: PiNativeOverlaySummary,
}

impl PreparedBootstrap {
    fn into_summary(self) -> BootstrapSummary {
        BootstrapSummary {
            inventory: self.state.inventory_summary(),
            runtime: self.runtime,
            logs: self.logger.summary(),
            state: self.state,
            claude_native: self.claude_native,
            codex_native: self.codex_native,
            pi_native: self.pi_native,
        }
    }
}

#[derive(Debug)]
pub enum RunError {
    Config(ConfigError),
    Runtime(crate::runtime::RuntimeError),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::Runtime(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for RunError {}

impl From<ConfigError> for RunError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<crate::runtime::RuntimeError> for RunError {
    fn from(error: crate::runtime::RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

pub fn run(cli: Cli) -> Result<RunOutcome, RunError> {
    let paths = resolve_paths(cli.config_file.as_deref(), cli.log_dir.as_deref())?;
    if let Some(outcome) = maybe_handle_utility_command(&cli, &paths)? {
        return Ok(outcome);
    }

    let prepared = prepare_bootstrap(&cli, paths)?;

    println!("Foreman bootstrap complete.");
    Ok(RunOutcome::Bootstrapped(Box::new(prepared.into_summary())))
}

pub fn run_main(cli: Cli) -> Result<(), RunError> {
    if cli.bootstrap_only {
        run(cli)?;
        return Ok(());
    }

    let paths = resolve_paths(cli.config_file.as_deref(), cli.log_dir.as_deref())?;
    if maybe_handle_utility_command(&cli, &paths)?.is_some() {
        return Ok(());
    }

    let prepared = prepare_bootstrap(&cli, paths)?;
    crate::runtime::run(prepared)?;
    Ok(())
}

fn maybe_handle_utility_command(
    cli: &Cli,
    paths: &crate::config::AppPaths,
) -> Result<Option<RunOutcome>, RunError> {
    if cli.config_path {
        println!("{}", paths.config_file.display());
        return Ok(Some(RunOutcome::PrintedConfigPath(
            paths.config_file.clone(),
        )));
    }

    if cli.init_config {
        write_default_config(&paths.config_file)?;
        println!(
            "Initialized default config at {}",
            paths.config_file.display()
        );
        return Ok(Some(RunOutcome::InitializedConfig(
            paths.config_file.clone(),
        )));
    }

    Ok(None)
}

pub(crate) fn prepare_bootstrap(
    cli: &Cli,
    paths: crate::config::AppPaths,
) -> Result<PreparedBootstrap, RunError> {
    let file_config = load_config(&paths.config_file)?;
    let runtime = RuntimeConfig::from_sources(paths, file_config, cli);

    let mut logger = RunLogger::start(
        &runtime.log_dir,
        runtime.log_retention,
        runtime.log_verbosity,
    )
    .map_err(ConfigError::Io)?;
    logger.log_bootstrap(&runtime).map_err(ConfigError::Io)?;
    logger
        .debug("bootstrap_debug_logging_enabled")
        .map_err(ConfigError::Io)?;
    let (state, claude_native, codex_native, pi_native) = bootstrap_state(&runtime);
    let inventory = state.inventory_summary();
    logger.log_inventory(&inventory).map_err(ConfigError::Io)?;
    logger
        .log_system_stats(&state.system_stats)
        .map_err(ConfigError::Io)?;
    if let Some(error) = &state.startup_error {
        logger.log_tmux_error(error).map_err(ConfigError::Io)?;
    }
    if let Some(alert) = &state.operator_alert {
        logger.log_operator_alert(alert).map_err(ConfigError::Io)?;
    }
    logger
        .log_claude_native_summary(&claude_native)
        .map_err(ConfigError::Io)?;
    for warning in &claude_native.warnings {
        logger
            .log_claude_native_warning(warning)
            .map_err(ConfigError::Io)?;
    }
    logger
        .log_codex_native_summary(&codex_native)
        .map_err(ConfigError::Io)?;
    for warning in &codex_native.warnings {
        logger
            .log_codex_native_warning(warning)
            .map_err(ConfigError::Io)?;
    }
    logger
        .log_pi_native_summary(&pi_native)
        .map_err(ConfigError::Io)?;
    for warning in &pi_native.warnings {
        logger
            .log_pi_native_warning(warning)
            .map_err(ConfigError::Io)?;
    }

    Ok(PreparedBootstrap {
        runtime,
        logger,
        state,
        claude_native,
        codex_native,
        pi_native,
    })
}

pub(crate) fn bootstrap_state(
    runtime: &RuntimeConfig,
) -> (
    AppState,
    ClaudeNativeOverlaySummary,
    CodexNativeOverlaySummary,
    PiNativeOverlaySummary,
) {
    let system_stats = SystemStatsService::new(SysinfoSystemStatsBackend::new())
        .snapshot()
        .unwrap_or_default();
    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(runtime.tmux_socket.clone()));
    match adapter.load_inventory(runtime.capture_lines) {
        Ok(mut inventory) => {
            let claude_native = apply_configured_claude_signals(
                &mut inventory,
                runtime.claude_native_dir.as_deref(),
                runtime.claude_integration_preference,
            );
            let codex_native = apply_configured_codex_signals(
                &mut inventory,
                runtime.codex_native_dir.as_deref(),
                runtime.codex_integration_preference,
            );
            let pi_native = apply_configured_pi_signals(
                &mut inventory,
                runtime.pi_native_dir.as_deref(),
                runtime.pi_integration_preference,
            );

            let mut state = AppState {
                popup_mode: runtime.popup,
                system_stats,
                ..AppState::with_inventory(inventory)
            };
            state.notifications.muted = !runtime.notifications_enabled;
            state.notifications.profile = runtime.notification_profile;
            state.notifications.cooldown_ticks = runtime.notification_cooldown_ticks;
            (state, claude_native, codex_native, pi_native)
        }
        Err(error) => {
            let mut state = AppState {
                popup_mode: runtime.popup,
                system_stats,
                operator_alert: Some(OperatorAlert::new(
                    OperatorAlertSource::Tmux,
                    OperatorAlertLevel::Warn,
                    format!("tmux unavailable: {error}"),
                )),
                startup_error: Some(error.to_string()),
                ..AppState::default()
            };
            state.notifications.muted = !runtime.notifications_enabled;
            state.notifications.profile = runtime.notification_profile;
            state.notifications.cooldown_ticks = runtime.notification_cooldown_ticks;
            (
                state,
                ClaudeNativeOverlaySummary::default(),
                CodexNativeOverlaySummary::default(),
                PiNativeOverlaySummary::default(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run, Cli, RunOutcome};
    use crate::app::OperatorAlertSource;
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

    #[test]
    fn bootstrap_respects_no_notify_runtime_flag() {
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
            "--no-notify",
        ]);

        let outcome = run(cli).expect("run should succeed");

        match outcome {
            RunOutcome::Bootstrapped(summary) => {
                assert!(summary.state.notifications.muted);
            }
            other => panic!("expected bootstrapped outcome, got {other:?}"),
        }
    }

    #[test]
    fn bootstrap_surfaces_tmux_unavailable_as_operator_alert() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let missing_socket = temp_dir.path().join("missing.sock");
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
            "--tmux-socket",
            missing_socket.to_str().expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("run should succeed");

        match outcome {
            RunOutcome::Bootstrapped(summary) => {
                assert!(summary.state.startup_error.is_some());
                let alert = summary
                    .state
                    .operator_alert
                    .as_ref()
                    .expect("tmux failure should surface an operator alert");
                assert_eq!(alert.source, OperatorAlertSource::Tmux);
                assert!(alert.message.contains("tmux unavailable"));
                assert!(summary.state.system_stats.cpu_pressure_percent.is_some());
                assert!(summary.state.system_stats.memory_pressure_percent.is_some());
            }
            other => panic!("expected bootstrapped outcome, got {other:?}"),
        }
    }
}
