use crate::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use crate::app::{
    AppState, InventorySummary, OperatorAlert, OperatorAlertLevel, OperatorAlertSource,
};
use crate::config::{
    load_config, resolve_paths, write_default_config, AppConfig, ConfigError, RuntimeConfig,
};
use crate::doctor::{
    collect_report, primary_runtime_alert_finding, runtime_findings, DoctorFixMode, DoctorOptions,
    DoctorReport, DoctorSeverity, RuntimeDoctorContext, SetupProviderSelection,
    SetupScopeSelection,
};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::logging::{RunLogSummary, RunLogger};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
use clap::Parser;
use std::fmt;
use std::path::{Path, PathBuf};

const CLI_ABOUT: &str = "Foreman - watch and control AI coding agents running in tmux.";
const CLI_LONG_ABOUT: &str = "Foreman is a keyboard-first dashboard for AI coding agents running in tmux.\n\nUse when: you want one operator surface for Claude Code, Codex, Pi, Gemini, or OpenCode panes.\nDon't use to: inspect a plain tmux tree with no agent workflow.";
const CLI_AFTER_HELP: &str = "First-time setup:\n  foreman --setup --user --project\n  foreman --doctor\n  foreman\n\nPreview setup changes:\n  foreman --setup --user --project --dry-run\n\nScoped installs:\n  foreman --setup --user\n  foreman --setup --project --codex --repo /path/to/repo\n\nAnother repo:\n  foreman --setup --project --repo /path/to/repo\n  foreman --doctor --repo /path/to/repo";

#[derive(Debug, Parser, Clone)]
#[command(
    name = "foreman",
    about = CLI_ABOUT,
    long_about = CLI_LONG_ABOUT,
    after_help = CLI_AFTER_HELP,
    next_line_help = true
)]
pub struct Cli {
    #[arg(
        long,
        conflicts_with_all = ["config_path", "init_config", "doctor"],
        help_heading = "Setup and Diagnosis",
        help = "Initialize Foreman config and apply safe hook/setup fixes. Defaults to project-level setup for the current repo, or user-level setup if no repo is detected."
    )]
    pub setup: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Preview what --setup would write without changing any files."
    )]
    pub dry_run: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Apply user-level provider wiring under your home directory."
    )]
    pub user: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Apply project-level provider wiring in the target repo. This is the default when a repo is detected."
    )]
    pub project: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Limit setup to Claude Code hook wiring."
    )]
    pub claude: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Limit setup to Codex hook wiring."
    )]
    pub codex: bool,

    #[arg(
        long,
        requires = "setup",
        help_heading = "Setup and Diagnosis",
        help = "Limit setup to Pi extension wiring."
    )]
    pub pi: bool,

    #[arg(
        long,
        conflicts_with_all = ["init_config", "doctor", "setup"],
        help_heading = "Utilities",
        help = "Print the resolved config path and exit."
    )]
    pub config_path: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "doctor", "setup"],
        help_heading = "Utilities",
        help = "Write the default config file if it does not exist, then exit."
    )]
    pub init_config: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "init_config", "setup"],
        help_heading = "Setup and Diagnosis",
        help = "Inspect config, hook wiring, and live tmux fallback. Defaults to the current repo when the working directory looks like one."
    )]
    pub doctor: bool,

    #[arg(
        long,
        requires = "doctor",
        help_heading = "Setup and Diagnosis",
        help = "Print doctor findings as JSON."
    )]
    pub doctor_json: bool,

    #[arg(
        long,
        requires = "doctor",
        help_heading = "Setup and Diagnosis",
        help = "Exit non-zero if doctor finds blocking issues."
    )]
    pub doctor_strict: bool,

    #[arg(long, requires = "doctor", hide = true)]
    pub doctor_fix: bool,

    #[arg(long, requires = "doctor_fix", hide = true)]
    pub doctor_dry_run: bool,

    #[arg(
        long = "repo",
        alias = "doctor-repo",
        value_name = "REPO",
        help_heading = "Setup and Diagnosis",
        help = "Repo to diagnose or set up. Defaults to the current directory when it looks like a repo."
    )]
    pub repo: Option<PathBuf>,

    #[arg(
        long,
        help_heading = "Runtime",
        help = "Use a specific config file instead of the default path."
    )]
    pub config_file: Option<PathBuf>,

    #[arg(
        long,
        help_heading = "Runtime",
        help = "Override the tmux inventory poll interval in milliseconds."
    )]
    pub poll_interval_ms: Option<u64>,

    #[arg(
        long,
        help_heading = "Runtime",
        help = "Override the number of tmux capture lines kept per pane."
    )]
    pub capture_lines: Option<usize>,

    #[arg(
        long,
        help_heading = "Runtime",
        help = "Close the tmux popup after a successful jump-to-pane action."
    )]
    pub popup: bool,

    #[arg(
        long = "no-notify",
        help_heading = "Runtime",
        help = "Mute desktop notifications for this run."
    )]
    pub no_notify: bool,

    #[arg(
        long,
        help_heading = "Runtime",
        help = "Enable debug logging for this run."
    )]
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
    Doctor(Box<DoctorReport>),
    Setup(Box<DoctorReport>),
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SetupDisplay {
    command: String,
    providers: String,
}

#[derive(Debug)]
pub enum RunError {
    Config(ConfigError),
    Doctor(crate::doctor::DoctorError),
    Runtime(crate::runtime::RuntimeError),
    Usage(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::Doctor(error) => write!(f, "{error}"),
            Self::Runtime(error) => write!(f, "{error}"),
            Self::Usage(message) => write!(f, "{message}"),
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

impl From<crate::doctor::DoctorError> for RunError {
    fn from(error: crate::doctor::DoctorError) -> Self {
        Self::Doctor(error)
    }
}

pub fn run(cli: Cli) -> Result<RunOutcome, RunError> {
    if cli.repo.is_some() && !cli.setup && !cli.doctor {
        return Err(RunError::Usage(
            "--repo is only supported with --setup or --doctor".to_string(),
        ));
    }

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

    if cli.setup {
        let (file_config, config_parse_error) = load_repair_config(&paths.config_file)?;
        let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
        let setup_scopes = SetupScopeSelection {
            user: cli.user,
            project: cli.project,
        };
        let setup_providers = SetupProviderSelection {
            claude: cli.claude,
            codex: cli.codex,
            pi: cli.pi,
        };
        let report = collect_report(
            &runtime,
            &DoctorOptions {
                repo_path: cli.repo.clone(),
                fix_mode: if cli.dry_run {
                    DoctorFixMode::DryRun
                } else {
                    DoctorFixMode::Apply
                },
                setup_scopes,
                setup_providers,
                config_parse_error,
            },
        )?;
        let setup_display = render_setup_display(
            setup_scopes,
            setup_providers,
            &cli.repo,
            report.repo_path.as_deref(),
        );
        println!(
            "{}",
            report.to_setup_text(
                cli.dry_run,
                &setup_display.command,
                setup_scopes.resolved(report.repo_path.is_some()),
                &setup_display.providers,
            )
        );
        return Ok(Some(RunOutcome::Setup(Box::new(report))));
    }

    if cli.doctor {
        let (file_config, config_parse_error) = load_repair_config(&paths.config_file)?;
        let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
        let fix_mode = if cli.doctor_fix {
            if cli.doctor_dry_run {
                DoctorFixMode::DryRun
            } else {
                DoctorFixMode::Apply
            }
        } else {
            DoctorFixMode::ReportOnly
        };
        let report = collect_report(
            &runtime,
            &DoctorOptions {
                repo_path: cli.repo.clone(),
                fix_mode,
                setup_scopes: SetupScopeSelection::default(),
                setup_providers: SetupProviderSelection::default(),
                config_parse_error,
            },
        )?;

        if cli.doctor_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(crate::doctor::DoctorError::from)?
            );
        } else {
            println!("{}", report.to_text());
        }

        if cli.doctor_strict && report.has_blocking_findings() {
            return Err(RunError::Doctor(crate::doctor::DoctorError::Io(
                std::io::Error::other(format!(
                    "doctor found {} blocking issue(s)",
                    report.blocking_findings().count()
                )),
            )));
        }

        return Ok(Some(RunOutcome::Doctor(Box::new(report))));
    }

    Ok(None)
}

fn load_repair_config(path: &Path) -> Result<(AppConfig, Option<String>), ConfigError> {
    match load_config(path) {
        Ok(config) => Ok((config, None)),
        Err(ConfigError::ParseToml(error)) => Ok((AppConfig::default(), Some(error.to_string()))),
        Err(error) => Err(error),
    }
}

fn render_setup_display(
    scopes: SetupScopeSelection,
    providers: SetupProviderSelection,
    explicit_repo: &Option<PathBuf>,
    resolved_repo: Option<&Path>,
) -> SetupDisplay {
    let resolved_scopes = scopes.resolved(resolved_repo.is_some());
    let mut command = String::from("foreman --setup");
    if resolved_scopes.user {
        command.push_str(" --user");
    }
    if resolved_scopes.project {
        command.push_str(" --project");
    }
    if providers.has_explicit_selection() {
        for flag in providers.selected_flags() {
            command.push(' ');
            command.push_str(flag);
        }
    }
    if let Some(repo) = explicit_repo.as_deref() {
        command.push_str(" --repo ");
        command.push_str(&repo.display().to_string());
    }

    SetupDisplay {
        command,
        providers: providers.selected_labels().join(", "),
    }
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
            state.runtime_diagnostics = runtime_findings(&RuntimeDoctorContext {
                runtime,
                config_exists: runtime.config_file.exists(),
                inventory: &state.inventory,
                claude_native: &claude_native,
                codex_native: &codex_native,
                pi_native: &pi_native,
            });
            if state.operator_alert.is_none() {
                if let Some(finding) = primary_runtime_alert_finding(&state.runtime_diagnostics) {
                    let level = match finding.severity {
                        DoctorSeverity::Error => OperatorAlertLevel::Error,
                        DoctorSeverity::Warn => OperatorAlertLevel::Warn,
                        DoctorSeverity::Ok | DoctorSeverity::Info => OperatorAlertLevel::Info,
                    };
                    state.operator_alert = Some(OperatorAlert::new(
                        OperatorAlertSource::Diagnostics,
                        level,
                        finding.summary.clone(),
                    ));
                }
            }
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

    #[test]
    fn setup_returns_setup_outcome_and_writes_safe_repo_files() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(temp_dir.path().join(".git")).expect("git dir should exist");
        let cli = Cli::parse_from([
            "foreman",
            "--setup",
            "--repo",
            temp_dir.path().to_str().expect("utf-8 path"),
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("setup should succeed");

        match outcome {
            RunOutcome::Setup(report) => {
                assert!(report
                    .fixes
                    .iter()
                    .any(|fix| fix.message == "initialize default config"));
                assert!(temp_dir.path().join(".claude/settings.local.json").exists());
                assert!(temp_dir.path().join(".codex/hooks.json").exists());
                assert!(temp_dir.path().join(".pi/extensions/foreman.ts").exists());
            }
            other => panic!("expected setup outcome, got {other:?}"),
        }
    }
}
