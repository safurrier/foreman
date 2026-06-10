use crate::adapters::tmux::{InventoryLoadMetrics, SystemTmuxBackend, TmuxAdapter, TmuxTarget};
use crate::app::{
    AppState, InventorySummary, OperatorAlert, OperatorAlertLevel, OperatorAlertSource,
    SelectionTarget,
};
use crate::config::{
    load_config, resolve_paths, write_default_config, AppConfig, ConfigError, RuntimeConfig,
};
use crate::doctor::{
    collect_report, primary_runtime_alert_finding, runtime_findings, DoctorArea, DoctorFinding,
    DoctorFixMode, DoctorOptions, DoctorReport, DoctorSeverity, RuntimeDoctorContext,
    SetupProviderSelection, SetupScopeSelection,
};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use crate::services::control_api::{
    agents_response, attach_extension_cards, attach_pull_request, focus_response, send_response,
    DisplayActivationResponse,
};
use crate::services::linked_repositories::{
    linked_repositories_file, load_linked_repositories, resolve_git_repository,
    save_linked_repositories,
};
use crate::services::logging::{RunLogSummary, RunLogger};
use crate::services::pull_requests::{
    PullRequestLookup, PullRequestService, SystemPullRequestBackend,
};
use crate::services::startup_cache::{load_startup_cache, StartupCacheLoadResult};
use crate::services::system_stats::{SysinfoSystemStatsBackend, SystemStatsService};
use crate::services::ui_preferences::{
    apply_preferences_to_runtime, load_ui_preferences, reset_ui_preferences, PersistedUiPreferences,
};
use crate::source_companion::{
    CompanionAction, CompanionRequest, CompanionResponse, SOURCE_COMPANION_PROTOCOL_VERSION,
};
use crate::source_companion_connect::{connect_ssh, ConnectSshConfig};
use crate::source_snapshots::{
    SourceRegistrationEnvelope, SourceSnapshotEnvelope, SourceSnapshotStore,
};
use crate::sources::{
    CompanionSource, ForemanSource, LocalSource, SnapshotSource, SourceConfig, SourceDescriptor,
    SourceError, SourceId, SourceScope, SourcesConfig, SshSource,
};
use clap::{Args, Parser, Subcommand};
use std::fmt;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Instant;

const CLI_ABOUT: &str = "Foreman - watch and control AI coding agents running in tmux.";
const CLI_LONG_ABOUT: &str = "Foreman is a keyboard-first dashboard for AI coding agents running in tmux.\n\nUse when: you want one operator surface for Claude Code, Codex, Pi, Gemini, or OpenCode panes.\nDon't use to: inspect a plain tmux tree with no agent workflow.";
const CLI_AFTER_HELP: &str = "First-time setup:\n  foreman --setup --user --project\n  foreman --doctor\n  foreman\n\nInspect resolved paths/config:\n  foreman --config-show\n\nPreview safe doctor fixes:\n  foreman --doctor --doctor-fix --doctor-dry-run\n\nPreview setup changes:\n  foreman --setup --user --project --dry-run\n\nScoped installs:\n  foreman --setup --user\n  foreman --setup --project --codex --repo /path/to/repo\n\nAnother repo:\n  foreman --setup --project --repo /path/to/repo\n  foreman --doctor --repo /path/to/repo";
const LINKS_ABOUT: &str = "Inspect and manage explicit pane-to-repository links.";
const LINKS_LONG_ABOUT: &str = "Repository links let Foreman associate a tmux pane with a git repository different from the pane's current working directory.\n\nUse when: an agent is running from notes, Obsidian, scratch space, or a launcher directory, but PR/HK/provider state belongs to a code repo elsewhere.\nDon't use when: the pane is already running from the repository Foreman should inspect.\n\nForeman still displays the pane's real working directory as Workspace, but uses the linked repository for PR lookup, HK cards, and extension providers. Links are persisted beside Foreman's config and guarded by the pane working-directory fingerprint to avoid stale tmux pane-id reuse.";
const LINKS_AFTER_HELP: &str = "Agent workflow:\n  1. Identify the pane id. If you are inside the target tmux pane, run: echo \"$TMUX_PANE\"\n  2. If you are outside the pane, inspect candidates with: foreman agents --json\n  3. Identify the intended code repo with: git -C <candidate-path> rev-parse --show-toplevel\n  4. Link the pane to that repo: foreman links add --pane %82 --repo ~/git_repositories/foreman --json\n  5. Verify Foreman uses the linked target: foreman agents --json --extensions\n\nExamples:\n  foreman links list --json\n  foreman links add --pane %82 --repo ~/git_repositories/foreman --json\n  foreman links remove --pane %82 --json";

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
        conflicts_with_all = ["config_path", "config_show", "init_config", "doctor"],
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
        conflicts_with_all = ["config_show", "init_config", "doctor", "setup"],
        help_heading = "Utilities",
        help = "Print the resolved config path and exit."
    )]
    pub config_path: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "init_config", "doctor", "setup", "reset_ui_state"],
        help_heading = "Utilities",
        help = "Print resolved config, UI state, cache, and runtime values."
    )]
    pub config_show: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "config_show", "doctor", "setup", "reset_ui_state"],
        help_heading = "Utilities",
        help = "Write the default config file if it does not exist, then exit."
    )]
    pub init_config: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "config_show", "init_config", "doctor", "setup"],
        help_heading = "Utilities",
        help = "Remove persisted runtime UI choices such as sort, theme, filters, collapsed sessions, and selection."
    )]
    pub reset_ui_state: bool,

    #[arg(
        long,
        conflicts_with_all = ["config_path", "config_show", "init_config", "setup", "reset_ui_state"],
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

    #[arg(
        long,
        requires = "doctor",
        help_heading = "Setup and Diagnosis",
        help = "Apply safe doctor fixes such as missing config files, native dirs, and hook wiring."
    )]
    pub doctor_fix: bool,

    #[arg(
        long,
        requires = "doctor_fix",
        help_heading = "Setup and Diagnosis",
        help = "Preview safe doctor fixes without writing files."
    )]
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

    #[arg(long, hide = true, global = true)]
    pub tmux_socket: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help_heading = "Runtime",
        help = "Target a named tmux server as if using `tmux -L <name>`."
    )]
    pub tmux_server_name: Option<String>,

    #[arg(
        long,
        global = true,
        value_name = "SOURCE_ID",
        help_heading = "Runtime",
        help = "Limit control commands to one configured source."
    )]
    pub source: Option<String>,

    #[arg(
        long,
        global = true,
        value_name = "SCOPE",
        help_heading = "Runtime",
        help = "Source scope for inventory commands: all, current, or local."
    )]
    pub sources: Option<String>,

    #[arg(long, hide = true)]
    pub claude_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub codex_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub pi_native_dir: Option<PathBuf>,

    #[arg(long, hide = true)]
    pub bootstrap_only: bool,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

fn tmux_backend_from_cli(cli: &Cli) -> SystemTmuxBackend {
    let target = match (&cli.tmux_socket, &cli.tmux_server_name) {
        (Some(socket), _) => TmuxTarget::Socket(socket.clone()),
        (None, Some(server_name)) => TmuxTarget::ServerName(server_name.clone()),
        (None, None) => TmuxTarget::Default,
    };
    SystemTmuxBackend::with_target(target)
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum CliCommand {
    /// Print the current Foreman agent inventory as JSON for external clients.
    Agents {
        #[arg(long, help = "Print the current agent inventory as JSON.")]
        json: bool,
        #[arg(long, help = "Include non-agent tmux panes in the output.")]
        all_panes: bool,
        #[arg(
            long,
            help = "Attach best-effort GitHub pull request metadata for each workspace."
        )]
        pull_requests: bool,
        #[arg(
            long,
            help = "Attach read-only extension/provider cards for each workspace."
        )]
        extensions: bool,
    },
    /// Focus tmux on a pane.
    Focus {
        #[arg(
            long,
            value_name = "PANE_ID",
            help = "tmux pane id to focus, e.g. %42."
        )]
        pane: String,
        #[arg(long, help = "Print a JSON action result.")]
        json: bool,
    },
    #[command(
        about = LINKS_ABOUT,
        long_about = LINKS_LONG_ABOUT,
        after_help = LINKS_AFTER_HELP
    )]
    Links {
        #[command(subcommand)]
        command: Option<LinksCommand>,
    },
    /// Inspect and manage persisted Foreman inventory sources.
    Sources {
        #[command(subcommand)]
        command: SourcesCommand,
    },
    /// Run a local source companion server for live reverse-tunnel transports.
    Companion {
        #[command(subcommand)]
        command: CompanionCommand,
    },
    /// Internal non-recursive source probe used by remote source transports.
    #[command(hide = true)]
    SourceProbe {
        #[arg(long)]
        local_only: bool,
        #[command(subcommand)]
        command: SourceProbeCommand,
    },
    /// Print read-only extension/provider cards for one live tmux pane as JSON.
    Extensions {
        #[arg(
            long,
            value_name = "PANE_ID",
            help = "Live tmux pane id whose workspace target should be inspected, e.g. %42. Linked repository targets are applied by Foreman."
        )]
        pane: String,
        #[arg(long, help = "Print a JSON extension-card result.")]
        json: bool,
    },
    /// Send text to a pane and press Enter.
    Send {
        #[arg(
            long,
            value_name = "PANE_ID",
            help = "tmux pane id to send input to, e.g. %42."
        )]
        pane: String,
        #[arg(long, conflicts_with = "stdin", help = "Text to send to the pane.")]
        text: Option<String>,
        #[arg(long, help = "Read text to send from stdin.")]
        stdin: bool,
        #[arg(long, help = "Print a JSON action result.")]
        json: bool,
    },
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum LinksCommand {
    /// Link a live tmux pane to the git repository Foreman should inspect.
    #[command(
        long_about = "Link a live tmux pane to the git repository Foreman should inspect.\n\nUse when: the pane is running from notes, Obsidian, scratch space, or a launcher directory, but the relevant PR/HK/provider state belongs to a different code repo.\nDon't use when: the pane's working directory is already the repo Foreman should inspect.\n\nThe command validates that the pane is live, resolves --repo to a git root, records the pane working directory as a stale-id guard, and persists the link beside Foreman's config."
    )]
    Add(LinkArgs),
    /// Remove a persisted repository link for a tmux pane.
    #[command(
        alias = "rm",
        long_about = "Remove a persisted repository link for a tmux pane.\n\nUse when: the pane should go back to using its real working directory for PR/HK/provider lookup, or when a stale/manual link is no longer wanted."
    )]
    Remove(UnlinkArgs),
    /// List persisted repository links.
    #[command(
        long_about = "List persisted repository links and the linked-repositories.json path.\n\nUse before linking to inspect current state, or after linking to verify the saved mapping. This reads Foreman's config-area link file only; it does not mutate links or inspect providers."
    )]
    List {
        #[arg(long, help = "Print persisted links as JSON for agent workflows.")]
        json: bool,
    },
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum SourcesCommand {
    /// List configured sources.
    List {
        #[arg(long, help = "Print sources as JSON.")]
        json: bool,
    },
    /// Add or replace a source.
    Add {
        #[command(subcommand)]
        command: SourcesAddCommand,
    },
    /// Write a source-local snapshot of the current tmux inventory.
    Snapshot(SourcesSnapshotArgs),
    /// Continuously refresh a source-local snapshot.
    Prewarm(SourcesPrewarmArgs),
    /// Write a source companion registration/heartbeat file.
    Register(SourcesRegisterArgs),
    /// Remove a configured source.
    Remove {
        source_id: String,
        #[arg(long, help = "Print a JSON action result.")]
        json: bool,
    },
    /// Run source health checks.
    Doctor {
        source_id: String,
        #[arg(long, help = "Print a JSON health report.")]
        json: bool,
    },
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum SourcesAddCommand {
    /// Add an SSH-backed source.
    Ssh(SourcesAddSshArgs),
    /// Add a read-only snapshot-backed source.
    Snapshot(SourcesAddSnapshotArgs),
    /// Add a live companion-backed source.
    Companion(SourcesAddCompanionArgs),
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum CompanionCommand {
    /// Serve source inventory/actions on a local TCP endpoint.
    Serve(CompanionServeArgs),
    /// Connect this host to a remote SSH host through a supervised reverse tunnel.
    ConnectSsh(CompanionConnectSshArgs),
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesAddSshArgs {
    pub source_id: String,
    #[arg(long)]
    pub host: String,
    #[arg(long, default_value = "foreman")]
    pub foreman: String,
    #[arg(long)]
    pub tmux_server_name: Option<String>,
    #[arg(long)]
    pub tmux_socket: Option<PathBuf>,
    #[arg(long)]
    pub label: String,
    #[arg(long, default_value = "ssh")]
    pub ssh: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesAddSnapshotArgs {
    pub source_id: String,
    #[arg(long)]
    pub path: PathBuf,
    #[arg(long)]
    pub label: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesAddCompanionArgs {
    pub source_id: String,
    #[arg(long)]
    pub endpoint: String,
    #[arg(long)]
    pub label: String,
    #[arg(long, default_value_t = 1_000)]
    pub timeout_ms: u64,
    #[arg(long)]
    pub allow_send: bool,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub snapshot_path: Option<PathBuf>,
    #[arg(long)]
    pub registration_path: Option<PathBuf>,
    #[arg(long)]
    pub activation_command: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesSnapshotArgs {
    #[arg(long)]
    pub source_id: String,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub all_panes: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesPrewarmArgs {
    #[arg(long)]
    pub source_id: String,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, default_value_t = 2_000)]
    pub interval_ms: u64,
    #[arg(long)]
    pub iterations: Option<u64>,
    #[arg(long)]
    pub all_panes: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct SourcesRegisterArgs {
    #[arg(long)]
    pub source_id: String,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub label: String,
    #[arg(long)]
    pub snapshot_path: Option<PathBuf>,
    #[arg(long)]
    pub companion_endpoint: Option<String>,
    #[arg(long)]
    pub display_activation_command: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct CompanionServeArgs {
    #[arg(long, default_value = "127.0.0.1:0")]
    pub bind: String,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long, default_value = "local")]
    pub source_id: String,
    #[arg(long)]
    pub activation_command: Option<String>,
    #[arg(long, default_value_t = 2_000)]
    pub activation_timeout_ms: u64,
    #[arg(long)]
    pub allow_send: bool,
    #[arg(long)]
    pub all_panes: bool,
    #[arg(long)]
    pub max_requests: Option<u64>,
    #[arg(long)]
    pub ready_file: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct CompanionConnectSshArgs {
    pub host: String,
    #[arg(long, default_value = "workstation")]
    pub source_id: String,
    #[arg(long, default_value = "Workstation")]
    pub label: String,
    #[arg(long, default_value_t = 4040)]
    pub remote_port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub remote_bind_host: String,
    #[arg(long, default_value = "127.0.0.1:0")]
    pub local_bind: String,
    #[arg(long)]
    pub allow_send: bool,
    #[arg(long)]
    pub token: Option<String>,
    #[arg(long)]
    pub activation_command: Option<String>,
    #[arg(long, default_value_t = 2_000)]
    pub activation_timeout_ms: u64,
    #[arg(long, default_value = "foreman")]
    pub remote_foreman: String,
    #[arg(long)]
    pub remote_config_file: Option<PathBuf>,
    #[arg(long, default_value = "ssh")]
    pub ssh: String,
    #[arg(long = "extra-ssh-arg")]
    pub extra_ssh_args: Vec<String>,
    #[arg(long)]
    pub replace: bool,
    #[arg(long)]
    pub no_remote_config: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Subcommand, Clone, PartialEq, Eq)]
pub enum SourceProbeCommand {
    Agents {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        all_panes: bool,
    },
    Focus {
        #[arg(long)]
        pane: String,
        #[arg(long)]
        json: bool,
    },
    Send {
        #[arg(long)]
        pane: String,
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        json: bool,
    },
    Extensions {
        #[arg(long)]
        pane: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct LinkArgs {
    #[arg(
        long,
        value_name = "PANE_ID",
        help = "Live tmux pane id to link, e.g. %42. Inside the target pane, `echo $TMUX_PANE` prints this value."
    )]
    pub pane: String,
    #[arg(
        long,
        value_name = "REPO",
        help = "Git working tree Foreman should use for PR/HK/provider lookup. Foreman resolves this path with `git rev-parse --show-toplevel`."
    )]
    pub repo: PathBuf,
    #[arg(long, help = "Print a JSON action result.")]
    pub json: bool,
}

#[derive(Debug, Args, Clone, PartialEq, Eq)]
pub struct UnlinkArgs {
    #[arg(
        long,
        value_name = "PANE_ID",
        help = "tmux pane id to unlink, e.g. %42."
    )]
    pub pane: String,
    #[arg(long, help = "Print a JSON action result.")]
    pub json: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    PrintedConfigPath(PathBuf),
    PrintedConfigReadout(String),
    InitializedConfig(PathBuf),
    Doctor(Box<DoctorReport>),
    Setup(Box<DoctorReport>),
    ResetUiState { path: PathBuf, removed: bool },
    ControlJson(serde_json::Value),
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
    pub startup_cache_generated_at_ms: Option<u64>,
    pub pending_selection_restore: Option<SelectionTarget>,
    pub persistent_runtime_diagnostics: Vec<DoctorFinding>,
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

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct BootstrapNativeTiming {
    pub claude_ms: u128,
    pub codex_ms: u128,
    pub pi_ms: u128,
    pub total_ms: u128,
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
    if let Some(outcome) = maybe_handle_control_command(&cli, &paths)? {
        return Ok(outcome);
    }
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
    if maybe_handle_control_command(&cli, &paths)?.is_some() {
        return Ok(());
    }
    if maybe_handle_utility_command(&cli, &paths)?.is_some() {
        return Ok(());
    }

    let prepared = prepare_runtime_bootstrap(&cli, paths)?;
    crate::runtime::run(prepared)?;
    Ok(())
}

fn link_pane_to_repository(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    args: &LinkArgs,
) -> Result<RunOutcome, RunError> {
    let pane_id = crate::app::PaneId::new(args.pane.clone());
    let adapter = TmuxAdapter::new(tmux_backend_from_cli(cli));
    let inventory = adapter.load_inventory(1).map_err(|error| {
        RunError::Usage(format!(
            "failed to inspect tmux panes before linking: {error}"
        ))
    })?;
    let pane_working_dir = inventory
        .local_pane(&pane_id)
        .map(|pane| pane.working_dir.clone())
        .ok_or_else(|| {
            RunError::Usage(format!(
                "cannot link unknown pane {}; choose a live tmux pane id",
                pane_id.as_str()
            ))
        })?;
    let repo_root = resolve_git_repository(&args.repo).map_err(|error| {
        RunError::Usage(format!(
            "failed to resolve linked repository {}: {error}",
            args.repo.display()
        ))
    })?;
    let mut links = load_linked_repositories(&paths.config_file).map_err(ConfigError::Io)?;
    links.insert(pane_id.clone(), repo_root.clone(), pane_working_dir.clone());
    save_linked_repositories(&paths.config_file, &links).map_err(ConfigError::Io)?;
    let value = serde_json::json!({
        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
        "ok": true,
        "action": "link",
        "paneId": pane_id.as_str(),
        "linkedRepository": repo_root,
        "paneWorkingDir": pane_working_dir,
        "path": linked_repositories_file(&paths.config_file),
    });
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|error| {
                RunError::Usage(format!("failed to encode link JSON: {error}"))
            })?
        );
    } else {
        println!(
            "Linked pane {} to {}",
            pane_id.as_str(),
            repo_root.display()
        );
    }
    Ok(RunOutcome::ControlJson(value))
}

fn unlink_pane_repository(
    paths: &crate::config::AppPaths,
    args: &UnlinkArgs,
) -> Result<RunOutcome, RunError> {
    let pane_id = crate::app::PaneId::new(args.pane.clone());
    let mut links = load_linked_repositories(&paths.config_file).map_err(ConfigError::Io)?;
    let removed = links.remove(&pane_id);
    save_linked_repositories(&paths.config_file, &links).map_err(ConfigError::Io)?;
    let value = serde_json::json!({
        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
        "ok": true,
        "action": "unlink",
        "paneId": pane_id.as_str(),
        "removed": removed,
        "path": linked_repositories_file(&paths.config_file),
    });
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|error| {
                RunError::Usage(format!("failed to encode unlink JSON: {error}"))
            })?
        );
    } else if removed {
        println!("Unlinked pane {}", pane_id.as_str());
    } else {
        println!("Pane {} had no linked repository", pane_id.as_str());
    }
    Ok(RunOutcome::ControlJson(value))
}

fn list_repository_links(
    paths: &crate::config::AppPaths,
    json: bool,
) -> Result<RunOutcome, RunError> {
    let links = load_linked_repositories(&paths.config_file).map_err(ConfigError::Io)?;
    let path = linked_repositories_file(&paths.config_file);
    let entries: Vec<_> = links
        .links
        .iter()
        .map(|(pane_id, link)| {
            serde_json::json!({
                "paneId": pane_id.as_str(),
                "linkedRepository": &link.repository,
                "paneWorkingDir": &link.pane_working_dir,
            })
        })
        .collect();
    let value = serde_json::json!({
        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
        "ok": true,
        "action": "links.list",
        "path": path,
        "links": entries,
    });
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|error| {
                RunError::Usage(format!("failed to encode links list JSON: {error}"))
            })?
        );
    } else if links.links.is_empty() {
        println!("No linked repositories recorded.");
        println!("Path: {}", path.display());
    } else {
        println!("Linked repositories:");
        for (pane_id, link) in &links.links {
            match &link.pane_working_dir {
                Some(working_dir) => println!(
                    "  {} -> {} (pane cwd guard: {})",
                    pane_id.as_str(),
                    link.repository.display(),
                    working_dir.display()
                ),
                None => println!(
                    "  {} -> {} (no pane cwd guard recorded)",
                    pane_id.as_str(),
                    link.repository.display()
                ),
            }
        }
        println!("Path: {}", path.display());
    }
    Ok(RunOutcome::ControlJson(value))
}

fn local_agents_response(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    include_non_agents: bool,
) -> Result<crate::services::control_api::AgentsResponse, RunError> {
    let file_config = load_config(&paths.config_file)?;
    let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
    let (mut state, ..) = bootstrap_state(&runtime);
    apply_linked_repositories_to_state(&mut state, &runtime);
    Ok(agents_response(&state, include_non_agents))
}

fn save_app_config(path: &Path, config: &AppConfig) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
    }
    let contents = toml::to_string_pretty(config).map_err(ConfigError::SerializeToml)?;
    std::fs::write(path, contents).map_err(ConfigError::Io)?;
    Ok(())
}

fn source_scope(cli: &Cli, config: &SourcesConfig) -> Result<SourceScope, RunError> {
    if let Some(source) = &cli.source {
        SourceId::validate(source).map_err(RunError::Usage)?;
        return Ok(SourceScope::All);
    }
    match &cli.sources {
        Some(scope) => SourceScope::parse(scope).map_err(RunError::Usage),
        None => Ok(config.default_scope),
    }
}

fn build_source_provider(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    id: SourceId,
    config: SourceConfig,
    include_non_agents: bool,
    query_timeout_ms: u64,
) -> Box<dyn ForemanSource + Send + Sync> {
    match &config {
        SourceConfig::Local {
            tmux_server_name, ..
        } => {
            let descriptor = SourceDescriptor::new(&id, &config);
            let mut agents_cli = cli.clone();
            let agents_paths = paths.clone();
            let mut focus_cli = cli.clone();
            let mut send_cli = cli.clone();
            let mut extensions_cli = cli.clone();
            if cli.tmux_socket.is_none() && cli.tmux_server_name.is_none() {
                agents_cli.tmux_server_name = tmux_server_name.clone();
                focus_cli.tmux_server_name = tmux_server_name.clone();
                send_cli.tmux_server_name = tmux_server_name.clone();
                extensions_cli.tmux_server_name = tmux_server_name.clone();
            }
            let extensions_paths = paths.clone();
            Box::new(LocalSource::new(
                descriptor,
                Box::new(move || {
                    local_agents_response(&agents_cli, &agents_paths, include_non_agents).map_err(
                        |error| SourceError::new("source.local.failed", error.to_string(), true),
                    )
                }),
                Box::new(move |pane| {
                    let adapter = TmuxAdapter::new(tmux_backend_from_cli(&focus_cli));
                    let pane_id = crate::app::PaneId::new(pane.to_string());
                    adapter.focus_pane(&pane_id).map_err(|error| {
                        SourceError::new(
                            "source.local.focus-failed",
                            format!("failed to focus pane {}: {error}", pane_id.as_str()),
                            true,
                        )
                    })?;
                    Ok(focus_response(pane_id.as_str()))
                }),
                Box::new(move |pane, text| {
                    let adapter = TmuxAdapter::new(tmux_backend_from_cli(&send_cli));
                    let pane_id = crate::app::PaneId::new(pane.to_string());
                    let result = adapter.send_input(&pane_id, text).map_err(|error| {
                        SourceError::new(
                            "source.local.send-failed",
                            format!("failed to send input to pane {}: {error}", pane_id.as_str()),
                            true,
                        )
                    })?;
                    Ok(send_response(pane_id.as_str(), result.bytes_sent))
                }),
                Box::new(move |pane| {
                    let file_config =
                        load_config(&extensions_paths.config_file).map_err(|error| {
                            SourceError::new(
                                "source.local.extensions-failed",
                                error.to_string(),
                                true,
                            )
                        })?;
                    let runtime = RuntimeConfig::from_sources(
                        extensions_paths.clone(),
                        file_config,
                        &extensions_cli,
                    );
                    let (mut state, ..) = bootstrap_state(&runtime);
                    apply_linked_repositories_to_state(&mut state, &runtime);
                    let pane_id = crate::app::PaneId::new(pane.to_string());
                    extension_cards_response_for_pane(&state, &pane_id).map_err(|error| {
                        SourceError::new("source.local.extensions-failed", error.to_string(), true)
                    })
                }),
            ))
        }
        SourceConfig::Ssh { .. } => Box::new(SshSource::new(id, config, query_timeout_ms)),
        SourceConfig::Snapshot { .. } => Box::new(SnapshotSource::new(id, config)),
        SourceConfig::Companion { .. } => {
            Box::new(CompanionSource::new(id, config, include_non_agents))
        }
    }
}

fn configured_source_providers(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    config: &SourcesConfig,
    include_non_agents: bool,
) -> Result<Vec<Box<dyn ForemanSource + Send + Sync>>, RunError> {
    let scope = source_scope(cli, config)?;
    let mut providers = Vec::new();
    for (id, source_config) in config.enabled_sources() {
        let include = match scope {
            SourceScope::All => true,
            SourceScope::Current | SourceScope::Local => {
                id.as_str() == crate::sources::LOCAL_SOURCE_ID
            }
        } && cli
            .source
            .as_ref()
            .is_none_or(|requested| requested == id.as_str());
        if include {
            providers.push(build_source_provider(
                cli,
                paths,
                id,
                source_config,
                include_non_agents,
                config.query_timeout_ms,
            ));
        }
    }
    if providers.is_empty() {
        return Err(RunError::Usage(
            "no enabled sources matched the requested scope".to_string(),
        ));
    }
    Ok(providers)
}

fn run_source_action(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    pane: &str,
    send_text: Option<&str>,
) -> Result<crate::services::control_api::ActionResponse, RunError> {
    let file_config = load_config(&paths.config_file)?;
    let source_id = cli
        .source
        .clone()
        .unwrap_or_else(|| crate::sources::LOCAL_SOURCE_ID.to_string());
    SourceId::validate(&source_id).map_err(RunError::Usage)?;
    let source_id = SourceId::new(source_id);
    let source_config = file_config
        .sources
        .get(&source_id)
        .ok_or_else(|| RunError::Usage(format!("unknown source {}", source_id.as_str())))?;
    let source = build_source_provider(
        cli,
        paths,
        source_id.clone(),
        source_config,
        true,
        file_config.sources.query_timeout_ms,
    );
    let descriptor = source.descriptor();
    let mut response = match send_text {
        Some(text) => source.send(pane, text),
        None => source.focus(pane),
    }
    .map_err(|error| RunError::Usage(error.message))?;
    response.wrap_source(&descriptor);
    Ok(response)
}

fn handle_source_probe(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    command: &SourceProbeCommand,
) -> Result<RunOutcome, RunError> {
    match command {
        SourceProbeCommand::Agents { json: _, all_panes } => {
            let response = local_agents_response(cli, paths, *all_panes)?;
            let value = serde_json::to_value(&response).map_err(|error| {
                RunError::Usage(format!("failed to encode source probe JSON: {error}"))
            })?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|error| {
                    RunError::Usage(format!("failed to encode source probe JSON: {error}"))
                })?
            );
            Ok(RunOutcome::ControlJson(value))
        }
        SourceProbeCommand::Focus { pane, json: _ } => {
            let adapter = TmuxAdapter::new(tmux_backend_from_cli(cli));
            let pane_id = crate::app::PaneId::new(pane.clone());
            adapter.focus_pane(&pane_id).map_err(|error| {
                RunError::Usage(format!(
                    "failed to focus pane {}: {error}",
                    pane_id.as_str()
                ))
            })?;
            let response = focus_response(pane_id.as_str());
            let value = serde_json::to_value(&response).map_err(|error| {
                RunError::Usage(format!("failed to encode source probe JSON: {error}"))
            })?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|error| {
                    RunError::Usage(format!("failed to encode source probe JSON: {error}"))
                })?
            );
            Ok(RunOutcome::ControlJson(value))
        }
        SourceProbeCommand::Send {
            pane,
            stdin,
            json: _,
        } => {
            if !stdin {
                return Err(RunError::Usage(
                    "source-probe send requires --stdin".to_string(),
                ));
            }
            let mut input = String::new();
            std::io::stdin()
                .read_to_string(&mut input)
                .map_err(|error| RunError::Usage(format!("failed to read stdin: {error}")))?;
            let adapter = TmuxAdapter::new(tmux_backend_from_cli(cli));
            let pane_id = crate::app::PaneId::new(pane.clone());
            let result = adapter.send_input(&pane_id, &input).map_err(|error| {
                RunError::Usage(format!(
                    "failed to send input to pane {}: {error}",
                    pane_id.as_str()
                ))
            })?;
            let response = send_response(pane_id.as_str(), result.bytes_sent);
            let value = serde_json::to_value(&response).map_err(|error| {
                RunError::Usage(format!("failed to encode source probe JSON: {error}"))
            })?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response).map_err(|error| {
                    RunError::Usage(format!("failed to encode source probe JSON: {error}"))
                })?
            );
            Ok(RunOutcome::ControlJson(value))
        }
        SourceProbeCommand::Extensions { pane, json: _ } => {
            let file_config = load_config(&paths.config_file)?;
            let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
            let (mut state, ..) = bootstrap_state(&runtime);
            apply_linked_repositories_to_state(&mut state, &runtime);
            let pane_id = crate::app::PaneId::new(pane.clone());
            let value = extension_cards_response_for_pane(&state, &pane_id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|error| {
                    RunError::Usage(format!("failed to encode source probe JSON: {error}"))
                })?
            );
            Ok(RunOutcome::ControlJson(value))
        }
    }
}

fn write_source_snapshot(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    source_id: &str,
    output: &Path,
    all_panes: bool,
) -> Result<SourceSnapshotEnvelope, RunError> {
    SourceId::validate(source_id).map_err(RunError::Usage)?;
    let source_id = SourceId::new(source_id.to_string());
    let mut response = local_agents_response(cli, paths, all_panes)?;
    response.sources.clear();
    response.source_diagnostics.clear();
    response.partial_failure_count = 0;
    let envelope = SourceSnapshotEnvelope::new(&source_id, response, crate::sources::unix_ms_now());
    let store = SourceSnapshotStore::new(
        output
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".")),
    );
    store
        .publish_snapshot_to(output, &envelope)
        .map_err(|error| RunError::Usage(format!("failed to write source snapshot: {error}")))?;
    Ok(envelope)
}

fn source_action_response(
    value: serde_json::Value,
    json: bool,
    human: impl FnOnce(),
) -> Result<RunOutcome, RunError> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).map_err(|error| {
                RunError::Usage(format!("failed to encode source JSON: {error}"))
            })?
        );
    } else {
        human();
    }
    Ok(RunOutcome::ControlJson(value))
}

fn handle_sources_command(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    command: &SourcesCommand,
) -> Result<RunOutcome, RunError> {
    let mut config = load_config(&paths.config_file)?;
    match command {
        SourcesCommand::List { json } => {
            let sources: Vec<_> = config
                .sources
                .enabled_sources()
                .into_iter()
                .map(|(id, source_config)| SourceDescriptor::new(&id, &source_config))
                .collect();
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "sources": sources,
                "path": paths.config_file,
            });
            source_action_response(value, *json, || {
                println!("Configured sources:");
                for (id, source_config) in config.sources.enabled_sources() {
                    println!("  {} ({})", id.as_str(), source_config.label());
                }
            })
        }
        SourcesCommand::Add { command } => {
            let (source_id, source_config, json) = match command {
                SourcesAddCommand::Ssh(args) => {
                    SourceId::validate(&args.source_id).map_err(RunError::Usage)?;
                    (
                        args.source_id.clone(),
                        SourceConfig::Ssh {
                            label: args.label.clone(),
                            host: args.host.clone(),
                            foreman: args.foreman.clone(),
                            tmux_server_name: args.tmux_server_name.clone(),
                            tmux_socket: args.tmux_socket.clone(),
                            ssh: args.ssh.clone(),
                            enabled: true,
                            query_timeout_ms: None,
                            extra_ssh_args: Vec::new(),
                            display: crate::sources::SourceDisplayConfig::default(),
                            jump: crate::sources::SourceJumpConfig::default(),
                        },
                        args.json,
                    )
                }
                SourcesAddCommand::Snapshot(args) => {
                    SourceId::validate(&args.source_id).map_err(RunError::Usage)?;
                    (
                        args.source_id.clone(),
                        SourceConfig::Snapshot {
                            label: args.label.clone(),
                            path: args.path.clone(),
                            enabled: true,
                            display: crate::sources::SourceDisplayConfig::default(),
                        },
                        args.json,
                    )
                }
                SourcesAddCommand::Companion(args) => {
                    SourceId::validate(&args.source_id).map_err(RunError::Usage)?;
                    (
                        args.source_id.clone(),
                        SourceConfig::Companion {
                            label: args.label.clone(),
                            endpoint: args.endpoint.clone(),
                            timeout_ms: args.timeout_ms,
                            enabled: true,
                            allow_send: args.allow_send,
                            token: args.token.clone(),
                            snapshot_path: args.snapshot_path.clone(),
                            registration_path: args.registration_path.clone(),
                            display: crate::sources::SourceDisplayConfig::default(),
                            jump: crate::sources::SourceJumpConfig {
                                activation_command: args.activation_command.clone(),
                            },
                        },
                        args.json,
                    )
                }
            };
            config
                .sources
                .entries
                .insert(source_id.clone(), source_config.clone());
            save_app_config(&paths.config_file, &config)?;
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": true,
                "action": "sources.add",
                "source": SourceDescriptor::new(&SourceId::new(source_id.clone()), &source_config),
                "path": paths.config_file,
            });
            source_action_response(value, json, || println!("Added source {source_id}"))
        }
        SourcesCommand::Snapshot(args) => {
            let envelope =
                write_source_snapshot(cli, paths, &args.source_id, &args.output, args.all_panes)?;
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": true,
                "action": "sources.snapshot",
                "sourceId": args.source_id,
                "path": args.output,
                "capturedAtUnixMs": envelope.captured_at_unix_ms,
                "expiresAtUnixMs": envelope.expires_at_unix_ms,
                "entryCount": envelope.response.entries.len(),
            });
            source_action_response(value, args.json, || {
                println!("Wrote source snapshot for {}", args.source_id)
            })
        }
        SourcesCommand::Prewarm(args) => {
            SourceId::validate(&args.source_id).map_err(RunError::Usage)?;
            let mut iterations = 0_u64;
            loop {
                let envelope = write_source_snapshot(
                    cli,
                    paths,
                    &args.source_id,
                    &args.output,
                    args.all_panes,
                )?;
                iterations += 1;
                if args.json {
                    eprintln!(
                        "{}",
                        serde_json::json!({
                            "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                            "action": "sources.prewarm.tick",
                            "sourceId": args.source_id,
                            "path": args.output,
                            "capturedAtUnixMs": envelope.captured_at_unix_ms,
                            "entryCount": envelope.response.entries.len(),
                            "iteration": iterations,
                        })
                    );
                }
                if args.iterations.is_some_and(|max| iterations >= max) {
                    let value = serde_json::json!({
                        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                        "ok": true,
                        "action": "sources.prewarm",
                        "sourceId": args.source_id,
                        "path": args.output,
                        "iterations": iterations,
                    });
                    return source_action_response(value, args.json, || {
                        println!(
                            "Prewarmed {} snapshot {} time(s)",
                            args.source_id, iterations
                        )
                    });
                }
                std::thread::sleep(std::time::Duration::from_millis(args.interval_ms.max(1)));
            }
        }
        SourcesCommand::Register(args) => {
            SourceId::validate(&args.source_id).map_err(RunError::Usage)?;
            let source_id = SourceId::new(args.source_id.clone());
            let now_ms = crate::sources::unix_ms_now();
            let mut registration =
                SourceRegistrationEnvelope::new(&source_id, args.label.clone(), now_ms);
            registration.snapshot_path = args.snapshot_path.clone();
            registration.companion_endpoint = args.companion_endpoint.clone();
            registration.tmux_server_name = cli.tmux_server_name.clone();
            registration.display_activation_command = args.display_activation_command.clone();
            let store = SourceSnapshotStore::new(
                args.output
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(".")),
            );
            store
                .publish_registration_to(&args.output, &registration)
                .map_err(|error| {
                    RunError::Usage(format!("failed to write source registration: {error}"))
                })?;
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": true,
                "action": "sources.register",
                "sourceId": args.source_id,
                "path": args.output,
                "heartbeatAtUnixMs": registration.heartbeat_at_unix_ms,
            });
            source_action_response(value, args.json, || {
                println!("Wrote source registration for {}", args.source_id)
            })
        }
        SourcesCommand::Remove { source_id, json } => {
            if source_id == crate::sources::LOCAL_SOURCE_ID {
                return Err(RunError::Usage(
                    "local source cannot be removed".to_string(),
                ));
            }
            let removed = config.sources.entries.remove(source_id).is_some();
            save_app_config(&paths.config_file, &config)?;
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": true,
                "action": "sources.remove",
                "sourceId": source_id,
                "removed": removed,
                "path": paths.config_file,
            });
            source_action_response(value, *json, || {
                println!("Removed source {source_id}: {removed}")
            })
        }
        SourcesCommand::Doctor { source_id, json } => {
            let source_id = SourceId::new(source_id.clone());
            let source_config = config
                .sources
                .get(&source_id)
                .ok_or_else(|| RunError::Usage(format!("unknown source {}", source_id.as_str())))?;
            let tmux_endpoint = source_tmux_endpoint_summary(&source_config);
            let source_metadata = source_metadata_summary(&source_id, &source_config);
            let provider = build_source_provider(
                &Cli::parse_from(["foreman", "agents"]),
                paths,
                source_id.clone(),
                source_config,
                true,
                config.sources.query_timeout_ms,
            );
            let descriptor = provider.descriptor();
            let started = std::time::Instant::now();
            let result = provider.agents();
            let duration_ms = started.elapsed().as_millis() as u64;
            let (ok, diagnostics) = match result {
                Ok(response) => (true, response.source_diagnostics),
                Err(error) => (
                    false,
                    vec![crate::sources::SourceDiagnostic::warning(
                        &descriptor,
                        error.code,
                        error.message,
                        error.retryable,
                        Some(duration_ms),
                    )],
                ),
            };
            let value = serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": ok,
                "source": descriptor,
                "durationMs": duration_ms,
                "tmuxEndpoint": tmux_endpoint,
                "metadata": source_metadata,
                "diagnostics": diagnostics,
            });
            source_action_response(value, *json, || {
                println!(
                    "Source {}: {}",
                    source_id.as_str(),
                    if ok { "ok" } else { "failed" }
                )
            })
        }
    }
}

fn source_metadata_summary(
    source_id: &SourceId,
    source_config: &SourceConfig,
) -> serde_json::Value {
    match source_config {
        SourceConfig::Companion {
            registration_path,
            snapshot_path,
            ..
        } => {
            let registration = registration_path.as_ref().map(|path| {
                let descriptor = SourceDescriptor::new(source_id, source_config);
                let store = SourceSnapshotStore::new(
                    path.parent()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from(".")),
                );
                match store.load_registration_for_source(
                    source_id,
                    &descriptor,
                    path,
                    crate::sources::unix_ms_now(),
                    15_000,
                ) {
                    Ok(loaded) => serde_json::json!({
                        "ok": true,
                        "path": path.display().to_string(),
                        "stale": loaded.stale,
                        "label": loaded.envelope.label,
                        "heartbeatAtUnixMs": loaded.envelope.heartbeat_at_unix_ms,
                        "snapshotPath": loaded.envelope.snapshot_path.as_ref().map(|path| path.display().to_string()),
                        "companionEndpoint": loaded.envelope.companion_endpoint,
                        "displayActivationCommand": loaded.envelope.display_activation_command,
                    }),
                    Err(error) => serde_json::json!({
                        "ok": false,
                        "path": path.display().to_string(),
                        "code": error.code,
                        "message": error.message,
                    }),
                }
            });
            serde_json::json!({
                "registration": registration,
                "snapshotPath": snapshot_path.as_ref().map(|path| path.display().to_string()),
            })
        }
        _ => serde_json::json!({}),
    }
}

fn handle_companion_command(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    command: &CompanionCommand,
) -> Result<RunOutcome, RunError> {
    match command {
        CompanionCommand::Serve(args) => serve_companion(cli, paths, args),
        CompanionCommand::ConnectSsh(args) => connect_ssh_command(cli, args),
    }
}

fn connect_ssh_command(cli: &Cli, args: &CompanionConnectSshArgs) -> Result<RunOutcome, RunError> {
    let config = ConnectSshConfig {
        host: args.host.clone(),
        source_id: args.source_id.clone(),
        label: args.label.clone(),
        remote_port: args.remote_port,
        remote_bind_host: args.remote_bind_host.clone(),
        local_bind: args.local_bind.clone(),
        allow_send: args.allow_send,
        token: args.token.clone(),
        activation_command: args.activation_command.clone(),
        activation_timeout_ms: args.activation_timeout_ms,
        remote_foreman: args.remote_foreman.clone(),
        remote_config_file: args.remote_config_file.clone(),
        ssh: args.ssh.clone(),
        extra_ssh_args: args.extra_ssh_args.clone(),
        replace: args.replace,
        no_remote_config: args.no_remote_config,
        json: args.json,
        config_file: cli.config_file.clone(),
        tmux_socket: cli.tmux_socket.clone(),
        tmux_server_name: cli.tmux_server_name.clone(),
    };
    let summary = connect_ssh(config).map_err(RunError::Usage)?;
    let value = serde_json::to_value(summary).map_err(|error| {
        RunError::Usage(format!("failed to encode connect-ssh summary: {error}"))
    })?;
    source_action_response(value, args.json, || {
        println!("Companion SSH connection ended")
    })
}

fn serve_companion(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    args: &CompanionServeArgs,
) -> Result<RunOutcome, RunError> {
    if args.allow_send && args.token.is_none() {
        return Err(RunError::Usage(
            "companion serve --allow-send requires --token".to_string(),
        ));
    }
    let listener = TcpListener::bind(&args.bind)
        .map_err(|error| RunError::Usage(format!("failed to bind companion endpoint: {error}")))?;
    let bound = listener.local_addr().map_err(|error| {
        RunError::Usage(format!("failed to inspect companion endpoint: {error}"))
    })?;
    if let Some(path) = &args.ready_file {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                RunError::Usage(format!("failed to create ready-file directory: {error}"))
            })?;
        }
        std::fs::write(path, bound.to_string()).map_err(|error| {
            RunError::Usage(format!("failed to write companion ready file: {error}"))
        })?;
    }
    if args.json {
        eprintln!(
            "{}",
            serde_json::json!({
                "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
                "ok": true,
                "action": "companion.serve.ready",
                "endpoint": bound.to_string(),
            })
        );
    } else {
        eprintln!("Foreman source companion listening on {bound}");
    }

    let mut handled = 0_u64;
    for stream in listener.incoming() {
        let mut stream = stream.map_err(|error| {
            RunError::Usage(format!("failed to accept companion request: {error}"))
        })?;
        let timeout = std::time::Duration::from_millis(5_000);
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            RunError::Usage(format!("failed to set companion read timeout: {error}"))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            RunError::Usage(format!("failed to set companion write timeout: {error}"))
        })?;
        let response = match read_companion_request(&stream)
            .and_then(|request| handle_companion_request(cli, paths, args, request))
        {
            Ok(response) => response,
            Err(response) => *response,
        };
        serde_json::to_writer(&mut stream, &response).map_err(|error| {
            RunError::Usage(format!("failed to write companion response: {error}"))
        })?;
        stream.write_all(b"\n").map_err(|error| {
            RunError::Usage(format!("failed to finish companion response: {error}"))
        })?;
        handled += 1;
        if args.max_requests.is_some_and(|max| handled >= max) {
            break;
        }
    }
    let value = serde_json::json!({
        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
        "ok": true,
        "action": "companion.serve",
        "endpoint": bound.to_string(),
        "handledRequests": handled,
    });
    source_action_response(value, args.json, || {
        println!("Companion served {handled} request(s) on {bound}")
    })
}

fn read_companion_request(
    stream: &std::net::TcpStream,
) -> Result<CompanionRequest, Box<CompanionResponse>> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| {
        Box::new(CompanionResponse::error(
            "unknown".to_string(),
            "source.companion.io",
            format!("failed to clone companion stream: {error}"),
        ))
    })?);
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|error| {
        Box::new(CompanionResponse::error(
            "unknown".to_string(),
            "source.companion.io",
            format!("failed to read companion request: {error}"),
        ))
    })?;
    let request: CompanionRequest = serde_json::from_str(&line).map_err(|error| {
        Box::new(CompanionResponse::error(
            "unknown".to_string(),
            "source.companion.invalid-json",
            format!("invalid companion request JSON: {error}"),
        ))
    })?;
    if request.protocol_version != SOURCE_COMPANION_PROTOCOL_VERSION {
        return Err(Box::new(CompanionResponse::error(
            request.request_id,
            "source.companion.protocol-unsupported",
            format!(
                "unsupported companion protocol version {}",
                request.protocol_version
            ),
        )));
    }
    Ok(request)
}

fn handle_companion_request(
    cli: &Cli,
    paths: &crate::config::AppPaths,
    args: &CompanionServeArgs,
    request: CompanionRequest,
) -> Result<CompanionResponse, Box<CompanionResponse>> {
    if args.token.as_ref() != request.token.as_ref() {
        return Err(Box::new(CompanionResponse::error(
            request.request_id,
            "source.companion.unauthorized",
            "companion request token did not match",
        )));
    }
    match request.action {
        CompanionAction::Agents => local_agents_response(cli, paths, request.all_panes)
            .map(|response| CompanionResponse::agents(request.request_id.clone(), response))
            .map_err(|error| {
                Box::new(CompanionResponse::error(
                    request.request_id,
                    "source.companion.agents-failed",
                    error.to_string(),
                ))
            }),
        CompanionAction::Focus => {
            let Some(pane) = request.pane_id.as_deref() else {
                return Err(Box::new(CompanionResponse::error(
                    request.request_id,
                    "source.companion.missing-pane",
                    "focus requires paneId",
                )));
            };
            let adapter = TmuxAdapter::new(tmux_backend_from_cli(cli));
            let pane_id = crate::app::PaneId::new(pane.to_string());
            adapter
                .focus_pane(&pane_id)
                .map(|_| {
                    let mut response = focus_response(pane);
                    response.display_activation = run_companion_activation(args, pane);
                    CompanionResponse::action(request.request_id.clone(), response)
                })
                .map_err(|error| {
                    Box::new(CompanionResponse::error(
                        request.request_id,
                        "source.companion.focus-failed",
                        error.to_string(),
                    ))
                })
        }
        CompanionAction::Send => {
            if !args.allow_send {
                return Err(Box::new(CompanionResponse::error(
                    request.request_id,
                    "source.companion.send-disabled",
                    "companion server has send disabled",
                )));
            }
            let Some(pane) = request.pane_id.as_deref() else {
                return Err(Box::new(CompanionResponse::error(
                    request.request_id,
                    "source.companion.missing-pane",
                    "send requires paneId",
                )));
            };
            let text = request.text.as_deref().unwrap_or_default();
            let adapter = TmuxAdapter::new(tmux_backend_from_cli(cli));
            let pane_id = crate::app::PaneId::new(pane.to_string());
            adapter
                .send_input(&pane_id, text)
                .map(|result| {
                    CompanionResponse::action(
                        request.request_id.clone(),
                        send_response(pane, result.bytes_sent),
                    )
                })
                .map_err(|error| {
                    Box::new(CompanionResponse::error(
                        request.request_id,
                        "source.companion.send-failed",
                        error.to_string(),
                    ))
                })
        }
        CompanionAction::Extensions => {
            let Some(pane) = request.pane_id.as_deref() else {
                return Err(Box::new(CompanionResponse::error(
                    request.request_id,
                    "source.companion.missing-pane",
                    "extensions requires paneId",
                )));
            };
            let file_config = load_config(&paths.config_file).map_err(|error| {
                Box::new(CompanionResponse::error(
                    request.request_id.clone(),
                    "source.companion.extensions-failed",
                    error.to_string(),
                ))
            })?;
            let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
            let (mut state, ..) = bootstrap_state(&runtime);
            apply_linked_repositories_to_state(&mut state, &runtime);
            let pane_id = crate::app::PaneId::new(pane.to_string());
            extension_cards_response_for_pane(&state, &pane_id)
                .map(|value| CompanionResponse::extensions(request.request_id.clone(), value))
                .map_err(|error| {
                    Box::new(CompanionResponse::error(
                        request.request_id,
                        "source.companion.extensions-failed",
                        error.to_string(),
                    ))
                })
        }
    }
}

fn run_companion_activation(
    args: &CompanionServeArgs,
    pane: &str,
) -> Option<DisplayActivationResponse> {
    let command = args.activation_command.as_deref()?;
    let timeout = std::time::Duration::from_millis(args.activation_timeout_ms.max(1));
    let mut child = match std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("FOREMAN_SOURCE_ID", &args.source_id)
        .env("FOREMAN_PANE_ID", pane)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return Some(DisplayActivationResponse {
                attempted: true,
                ok: false,
                message: Some(format!("failed to run activation command: {error}")),
            })
        }
    };

    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                let _ = child.wait_with_output();
                return Some(DisplayActivationResponse {
                    attempted: true,
                    ok: true,
                    message: None,
                });
            }
            Ok(Some(status)) => {
                let stderr = child
                    .wait_with_output()
                    .map(|output| String::from_utf8_lossy(&output.stderr).trim().to_string())
                    .unwrap_or_default();
                let message = if stderr.is_empty() {
                    format!("activation command exited with status {status}")
                } else {
                    format!("activation command exited with status {status}: {stderr}")
                };
                return Some(DisplayActivationResponse {
                    attempted: true,
                    ok: false,
                    message: Some(message),
                });
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Some(DisplayActivationResponse {
                    attempted: true,
                    ok: false,
                    message: Some(format!(
                        "activation command timed out after {}ms",
                        args.activation_timeout_ms.max(1)
                    )),
                });
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Some(DisplayActivationResponse {
                    attempted: true,
                    ok: false,
                    message: Some(format!("failed to wait for activation command: {error}")),
                });
            }
        }
    }
}

fn source_tmux_endpoint_summary(source_config: &SourceConfig) -> serde_json::Value {
    match source_config {
        SourceConfig::Local {
            tmux_server_name, ..
        } => serde_json::json!({
            "kind": "local",
            "configuredServerName": tmux_server_name,
            "configuredSocket": null,
            "note": "Local sources use the current tmux environment unless a server name or socket override is configured."
        }),
        SourceConfig::Ssh {
            tmux_server_name,
            tmux_socket,
            ..
        } => serde_json::json!({
            "kind": "ssh",
            "configuredServerName": tmux_server_name,
            "configuredSocket": tmux_socket.as_ref().map(|path| path.display().to_string()),
            "note": "Noninteractive SSH does not inherit the $TMUX value from an attached terminal tab; configure tmux_server_name or tmux_socket when the source should query a specific remote tmux endpoint."
        }),
        SourceConfig::Snapshot { path, .. } => serde_json::json!({
            "kind": "snapshot",
            "path": path.display().to_string(),
            "note": "Snapshot sources are read-only cached inventories. Focus/send require a live source transport."
        }),
        SourceConfig::Companion {
            endpoint,
            allow_send,
            snapshot_path,
            registration_path,
            ..
        } => serde_json::json!({
            "kind": "companion",
            "endpoint": endpoint,
            "allowSend": allow_send,
            "snapshotPath": snapshot_path.as_ref().map(|path| path.display().to_string()),
            "registrationPath": registration_path.as_ref().map(|path| path.display().to_string()),
            "note": "Companion sources use a live JSON-line TCP protocol, commonly through a reverse SSH tunnel. Send requires allow_send=true on the trusted source config and companion server."
        }),
    }
}

fn maybe_handle_control_command(
    cli: &Cli,
    paths: &crate::config::AppPaths,
) -> Result<Option<RunOutcome>, RunError> {
    let Some(command) = cli.command.as_ref() else {
        return Ok(None);
    };
    if let Some(flag) = control_command_conflict(cli) {
        return Err(RunError::Usage(format!(
            "control subcommands cannot be combined with {flag}"
        )));
    }

    match command {
        CliCommand::Agents {
            all_panes,
            pull_requests,
            extensions,
            ..
        } => {
            let file_config = load_config(&paths.config_file)?;
            let providers =
                configured_source_providers(cli, paths, &file_config.sources, *all_panes)?;
            let aggregate = crate::sources::SourceAggregator::new(providers).aggregate();
            let mut response = aggregate.response;
            if *pull_requests {
                attach_pull_requests_to_response(&mut response);
            }
            if *extensions {
                attach_extensions_to_response(&mut response);
            }
            let mut value = serde_json::to_value(&response).map_err(|error| {
                RunError::Usage(format!("failed to encode agents JSON: {error}"))
            })?;
            if let serde_json::Value::Object(object) = &mut value {
                object.insert(
                    "sources".to_string(),
                    serde_json::to_value(aggregate.sources).map_err(|error| {
                        RunError::Usage(format!("failed to encode sources JSON: {error}"))
                    })?,
                );
                object.insert(
                    "sourceDiagnostics".to_string(),
                    serde_json::to_value(aggregate.source_diagnostics).map_err(|error| {
                        RunError::Usage(format!(
                            "failed to encode source diagnostics JSON: {error}"
                        ))
                    })?,
                );
                object.insert(
                    "partialFailureCount".to_string(),
                    serde_json::json!(aggregate.partial_failure_count),
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&value).map_err(|error| {
                    RunError::Usage(format!("failed to encode agents JSON: {error}"))
                })?
            );
            Ok(Some(RunOutcome::ControlJson(value)))
        }
        CliCommand::Focus { pane, json } => {
            let response = run_source_action(cli, paths, pane, None)?;
            let value = serde_json::to_value(&response).map_err(|error| {
                RunError::Usage(format!("failed to encode focus JSON: {error}"))
            })?;
            if *json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response).map_err(|error| {
                        RunError::Usage(format!("failed to encode focus JSON: {error}"))
                    })?
                );
            } else {
                println!("Focused pane {pane}");
            }
            Ok(Some(RunOutcome::ControlJson(value)))
        }
        CliCommand::Links { command } => match command {
            Some(LinksCommand::Add(args)) => Ok(Some(link_pane_to_repository(cli, paths, args)?)),
            Some(LinksCommand::Remove(args)) => Ok(Some(unlink_pane_repository(paths, args)?)),
            Some(LinksCommand::List { json }) => Ok(Some(list_repository_links(paths, *json)?)),
            None => Ok(Some(list_repository_links(paths, false)?)),
        },
        CliCommand::Sources { command } => Ok(Some(handle_sources_command(cli, paths, command)?)),
        CliCommand::Companion { command } => {
            Ok(Some(handle_companion_command(cli, paths, command)?))
        }
        CliCommand::SourceProbe {
            local_only,
            command,
        } => {
            if !local_only {
                return Err(RunError::Usage(
                    "source-probe requires --local-only".to_string(),
                ));
            }
            Ok(Some(handle_source_probe(cli, paths, command)?))
        }
        CliCommand::Extensions { pane, json } if cli.source.is_some() => {
            let file_config = load_config(&paths.config_file)?;
            let source_id = SourceId::new(cli.source.clone().expect("source checked"));
            let source_config = file_config
                .sources
                .get(&source_id)
                .ok_or_else(|| RunError::Usage(format!("unknown source {}", source_id.as_str())))?;
            let source = build_source_provider(
                cli,
                paths,
                source_id,
                source_config,
                true,
                file_config.sources.query_timeout_ms,
            );
            let descriptor = source.descriptor();
            let mut value = source
                .extensions(pane)
                .map_err(|error| RunError::Usage(error.message))?;
            if let serde_json::Value::Object(object) = &mut value {
                object.insert("sourceId".to_string(), serde_json::json!(descriptor.id));
                object.insert(
                    "sourcePaneId".to_string(),
                    serde_json::json!(crate::sources::SourcePaneId::new(
                        SourceId::new(descriptor.id.clone()),
                        pane.clone()
                    )
                    .stable_id()),
                );
            }
            if *json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).map_err(|error| {
                        RunError::Usage(format!("failed to encode extension JSON: {error}"))
                    })?
                );
            } else {
                let count = value["extensionCards"].as_array().map_or(0, Vec::len);
                println!("Extension cards for {}: {}", pane, count);
            }
            Ok(Some(RunOutcome::ControlJson(value)))
        }
        CliCommand::Extensions { pane, json } => {
            let file_config = load_config(&paths.config_file)?;
            let runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
            let (mut state, ..) = bootstrap_state(&runtime);
            apply_linked_repositories_to_state(&mut state, &runtime);
            let pane_id = crate::app::PaneId::new(pane.clone());
            let value = extension_cards_response_for_pane(&state, &pane_id)?;
            if *json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).map_err(|error| {
                        RunError::Usage(format!("failed to encode extension JSON: {error}"))
                    })?
                );
            } else {
                let count = value["extensionCards"].as_array().map_or(0, Vec::len);
                println!("Extension cards for {}: {}", pane_id.as_str(), count);
            }
            Ok(Some(RunOutcome::ControlJson(value)))
        }
        CliCommand::Send {
            pane,
            text,
            stdin,
            json,
        } => {
            let input = if *stdin {
                let mut input = String::new();
                std::io::stdin()
                    .read_to_string(&mut input)
                    .map_err(|error| RunError::Usage(format!("failed to read stdin: {error}")))?;
                input
            } else {
                text.clone().ok_or_else(|| {
                    RunError::Usage("send requires --text <TEXT> or --stdin".to_string())
                })?
            };
            let response = run_source_action(cli, paths, pane, Some(&input))?;
            let value = serde_json::to_value(&response)
                .map_err(|error| RunError::Usage(format!("failed to encode send JSON: {error}")))?;
            if *json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&response).map_err(|error| {
                        RunError::Usage(format!("failed to encode send JSON: {error}"))
                    })?
                );
            } else {
                println!(
                    "Sent {} byte(s) to pane {}",
                    response.bytes_sent.unwrap_or(0),
                    response.pane_id
                );
            }
            Ok(Some(RunOutcome::ControlJson(value)))
        }
    }
}

fn extension_cards_response_for_pane(
    state: &AppState,
    pane_id: &crate::app::PaneId,
) -> Result<serde_json::Value, RunError> {
    extension_cards_response_for_pane_with_collector(state, pane_id, |target| {
        use crate::services::extensions::collect_workspace_extensions;
        collect_workspace_extensions(std::slice::from_ref(target))
            .remove(target)
            .unwrap_or_default()
    })
}

fn extension_cards_response_for_pane_with_collector<F>(
    state: &AppState,
    pane_id: &crate::app::PaneId,
    collect: F,
) -> Result<serde_json::Value, RunError>
where
    F: FnOnce(&PathBuf) -> Vec<crate::services::extensions::ControlExtensionCard>,
{
    let pane = state.inventory.local_pane(pane_id).ok_or_else(|| {
        RunError::Usage(format!(
            "cannot inspect extensions for unknown pane {}; choose a live tmux pane id",
            pane_id.as_str()
        ))
    })?;
    let target_path = state.workspace_target_for_pane(pane).ok_or_else(|| {
        RunError::Usage(format!(
            "cannot inspect extensions for pane {}; pane has no workspace target",
            pane_id.as_str()
        ))
    })?;
    let workspace = pane.working_dir.clone();
    let linked_repository = state.linked_repository_for_pane(pane).cloned();
    let cards = collect(&target_path);
    Ok(serde_json::json!({
        "schemaVersion": crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
        "ok": true,
        "action": "extensions",
        "paneId": pane_id.as_str(),
        "workspace": workspace,
        "linkedRepository": linked_repository,
        "targetPath": target_path,
        "extensionCards": cards,
    }))
}

fn entry_workspace_target(entry: &crate::services::control_api::AgentEntry) -> Option<PathBuf> {
    entry
        .linked_repository
        .as_ref()
        .or(entry.working_dir.as_ref())
        .map(PathBuf::from)
}

fn attach_extensions_to_response(response: &mut crate::services::control_api::AgentsResponse) {
    use crate::services::extensions::collect_workspace_extensions;

    let workspaces = response
        .entries
        .iter()
        .filter(|entry| entry.source_id == crate::sources::LOCAL_SOURCE_ID)
        .filter_map(entry_workspace_target)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let cache = collect_workspace_extensions(&workspaces);

    for entry in &mut response.entries {
        if entry.source_id != crate::sources::LOCAL_SOURCE_ID {
            continue;
        }
        let Some(path) = entry_workspace_target(entry) else {
            continue;
        };
        if let Some(cards) = cache.get(&path).filter(|cards| !cards.is_empty()) {
            attach_extension_cards(entry, cards.clone());
        }
    }
}

fn attach_pull_requests_to_response(response: &mut crate::services::control_api::AgentsResponse) {
    use std::collections::BTreeMap;

    let service = PullRequestService::new(SystemPullRequestBackend::new());
    let mut cache = BTreeMap::<PathBuf, PullRequestLookup>::new();

    for entry in &mut response.entries {
        if entry.source_id != crate::sources::LOCAL_SOURCE_ID {
            continue;
        }
        let Some(path) = entry_workspace_target(entry) else {
            continue;
        };
        let lookup = cache.entry(path.clone()).or_insert_with(|| {
            service
                .lookup(&path)
                .unwrap_or(PullRequestLookup::Unavailable {
                    message: "pull request lookup failed".to_string(),
                })
        });
        if let PullRequestLookup::Available(pull_request) = lookup {
            attach_pull_request(entry, pull_request);
        }
    }
}

fn control_command_conflict(cli: &Cli) -> Option<&'static str> {
    [
        ("--setup", cli.setup),
        ("--dry-run", cli.dry_run),
        ("--user", cli.user),
        ("--project", cli.project),
        ("--claude", cli.claude),
        ("--codex", cli.codex),
        ("--pi", cli.pi),
        ("--config-path", cli.config_path),
        ("--config-show", cli.config_show),
        ("--init-config", cli.init_config),
        ("--reset-ui-state", cli.reset_ui_state),
        ("--doctor", cli.doctor),
        ("--doctor-json", cli.doctor_json),
        ("--doctor-strict", cli.doctor_strict),
        ("--doctor-fix", cli.doctor_fix),
        ("--doctor-dry-run", cli.doctor_dry_run),
        ("--repo", cli.repo.is_some()),
    ]
    .into_iter()
    .find_map(|(flag, present)| present.then_some(flag))
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

    if cli.config_show {
        let readout = config_readout(cli, paths)?;
        println!("{readout}");
        return Ok(Some(RunOutcome::PrintedConfigReadout(readout)));
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

    if cli.reset_ui_state {
        let removed = reset_ui_preferences(&paths.ui_preferences_file).map_err(ConfigError::Io)?;
        if removed {
            println!(
                "Removed UI state at {}",
                paths.ui_preferences_file.display()
            );
        } else {
            println!(
                "No persisted UI state found at {}",
                paths.ui_preferences_file.display()
            );
        }
        return Ok(Some(RunOutcome::ResetUiState {
            path: paths.ui_preferences_file.clone(),
            removed,
        }));
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

fn config_readout(cli: &Cli, paths: &crate::config::AppPaths) -> Result<String, RunError> {
    let (file_config, config_parse_error) = load_repair_config(&paths.config_file)?;
    let mut runtime = RuntimeConfig::from_sources(paths.clone(), file_config, cli);
    let (ui_preferences, ui_preferences_error) = load_and_apply_ui_preferences(&mut runtime);
    let config_status = if paths.config_file.exists() {
        "present"
    } else {
        "missing"
    };
    let ui_status = if paths.ui_preferences_file.exists() {
        if ui_preferences_error.is_some() {
            "unreadable"
        } else {
            "present"
        }
    } else {
        "missing"
    };
    let cache_status = match load_startup_cache(&runtime, runtime.startup_cache_max_age_ms) {
        Ok(StartupCacheLoadResult::Loaded(cache)) => format!(
            "loaded path={} age_ms={} sessions={} panes={}",
            cache.path.display(),
            cache.age_ms,
            cache.inventory.session_count(),
            cache.inventory.pane_count()
        ),
        Ok(StartupCacheLoadResult::Stale {
            path,
            age_ms,
            inventory,
        }) => {
            format!(
                "stale path={} age_ms={age_ms} sessions={} panes={}",
                path.display(),
                inventory.sessions,
                inventory.panes
            )
        }
        Ok(StartupCacheLoadResult::Missing { path }) => format!("missing path={}", path.display()),
        Ok(StartupCacheLoadResult::Empty { path }) => format!("empty path={}", path.display()),
        Ok(StartupCacheLoadResult::Invalid { path, reason }) => {
            format!("invalid path={} reason={reason}", path.display())
        }
        Err(error) => format!("unavailable error={error}"),
    };
    let backends = runtime
        .notification_backends
        .iter()
        .map(|backend| backend.label())
        .collect::<Vec<_>>()
        .join(", ");

    let mut lines = vec![
        "Foreman config".to_string(),
        format!(
            "  config_file: {} ({config_status})",
            runtime.config_file.display()
        ),
        format!(
            "  ui_state: {} ({ui_status})",
            runtime.ui_preferences_file.display()
        ),
        format!("  log_dir: {}", runtime.log_dir.display()),
        format!(
            "  startup_cache_dir: {}",
            runtime.startup_cache_dir.display()
        ),
        format!("  startup_cache: {cache_status}"),
        format!(
            "  sort: {}{}",
            runtime.default_sort.config_label(),
            if runtime.ui_default_sort_configured {
                " (config)"
            } else {
                " (default or persisted)"
            }
        ),
        format!(
            "  theme: {}{}",
            runtime.theme.label(),
            if runtime.ui_theme_configured {
                " (config)"
            } else {
                " (default or persisted)"
            }
        ),
        format!(
            "  ui_state_values: sort={} theme={} filters={} collapsed_sessions={} selection={}",
            ui_preferences
                .sort_mode
                .map(|sort| sort.config_label())
                .unwrap_or("unset"),
            ui_preferences
                .theme
                .map(|theme| theme.label())
                .unwrap_or("unset"),
            ui_preferences.filters.is_some(),
            ui_preferences.collapsed_sessions.len(),
            ui_preferences.selection.is_some()
        ),
        format!(
            "  polling: {}ms capture_lines={}",
            runtime.poll_interval_ms, runtime.capture_lines
        ),
        format!(
            "  notifications: enabled={} profile={} sound_profile={} cooldown_ticks={} backends={}",
            runtime.notifications_enabled,
            runtime.notification_profile.label(),
            runtime.notification_sound_profile,
            runtime.notification_cooldown_ticks,
            backends
        ),
        format!(
            "  pull_requests: enabled={} poll_interval_ms={}",
            runtime.pull_request_monitoring_enabled, runtime.pull_request_poll_interval_ms
        ),
        format!(
            "  claude: mode={} native_dir={}",
            runtime.claude_integration_preference.label(),
            runtime
                .claude_native_dir
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "unset".to_string())
        ),
        format!(
            "  codex: mode={} native_dir={}",
            runtime.codex_integration_preference.label(),
            runtime
                .codex_native_dir
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "unset".to_string())
        ),
        format!(
            "  pi: mode={} native_dir={}",
            runtime.pi_integration_preference.label(),
            runtime
                .pi_native_dir
                .as_deref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "unset".to_string())
        ),
    ];
    if let Some(error) = config_parse_error {
        lines.push(format!("  config_parse_error: {error}"));
    }
    if let Some(error) = ui_preferences_error {
        lines.push(format!("  ui_state_error: {error}"));
    }
    lines.push("  next: foreman --doctor --doctor-fix --doctor-dry-run".to_string());
    Ok(lines.join("\n"))
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
    let mut runtime = RuntimeConfig::from_sources(paths, file_config, cli);
    let (ui_preferences, ui_preferences_error) = load_and_apply_ui_preferences(&mut runtime);

    let mut logger = start_run_logger(&runtime)?;
    log_ui_preferences_status(&mut logger, &runtime, ui_preferences_error.clone())?;
    let (mut state, tmux_metrics, native_timing, claude_native, codex_native, pi_native) =
        bootstrap_state(&runtime);
    apply_ui_preferences_to_state(&mut state, &runtime, &ui_preferences);
    apply_linked_repositories_to_state(&mut state, &runtime);
    apply_ui_preferences_error_to_state(&mut state, &runtime, ui_preferences_error.as_deref());
    logger
            .log_timing(
                "inventory_tmux",
                &format!(
                    "pane_records={} captures={} priority_captures={} deferred_captures={} forced_captures={} reused_previews={} capture_failures={} list_panes_ms={} capture_total_ms={} capture_max_ms={} total_ms={}",
                    tmux_metrics.pane_records,
                    tmux_metrics.capture_count,
                    tmux_metrics.priority_capture_count,
                    tmux_metrics.deferred_capture_count,
                    tmux_metrics.forced_capture_count,
                    tmux_metrics.reused_preview_count,
                    tmux_metrics.capture_failures,
                    tmux_metrics.list_panes_ms,
                    tmux_metrics.capture_total_ms,
                tmux_metrics.capture_max_ms,
                tmux_metrics.total_ms,
            ),
        )
        .map_err(ConfigError::Io)?;
    for pane_id in &tmux_metrics.capture_failure_panes {
        logger
            .log_tmux_capture_failure(pane_id.as_str())
            .map_err(ConfigError::Io)?;
    }
    logger
        .log_timing(
            "inventory_native",
            &format!(
                "claude_ms={} codex_ms={} pi_ms={} total_ms={} claude_applied={} claude_fallbacks={} codex_applied={} codex_fallbacks={} pi_applied={} pi_fallbacks={}",
                native_timing.claude_ms,
                native_timing.codex_ms,
                native_timing.pi_ms,
                native_timing.total_ms,
                claude_native.applied,
                claude_native.fallback_to_compatibility,
                codex_native.applied,
                codex_native.fallback_to_compatibility,
                pi_native.applied,
                pi_native.fallback_to_compatibility,
            ),
        )
        .map_err(ConfigError::Io)?;
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

    let persistent_runtime_diagnostics =
        ui_preferences_error_diagnostics(&runtime, ui_preferences_error.as_deref());

    Ok(PreparedBootstrap {
        runtime,
        logger,
        state,
        claude_native,
        codex_native,
        pi_native,
        startup_cache_generated_at_ms: None,
        pending_selection_restore: ui_preferences.selection.clone(),
        persistent_runtime_diagnostics,
    })
}

fn prepare_runtime_bootstrap(
    cli: &Cli,
    paths: crate::config::AppPaths,
) -> Result<PreparedBootstrap, RunError> {
    let file_config = load_config(&paths.config_file)?;
    let mut runtime = RuntimeConfig::from_sources(paths, file_config, cli);
    let (ui_preferences, ui_preferences_error) = load_and_apply_ui_preferences(&mut runtime);
    let mut logger = start_run_logger(&runtime)?;
    log_ui_preferences_status(&mut logger, &runtime, ui_preferences_error.clone())?;
    let system_stats = SystemStatsService::new(SysinfoSystemStatsBackend::new())
        .snapshot()
        .unwrap_or_default();
    let cache_load_started = Instant::now();
    let cache_enabled_for_scope = runtime.source.is_none();
    let cache_result = if runtime.popup && cache_enabled_for_scope {
        match load_startup_cache(&runtime, runtime.startup_cache_max_age_ms) {
            Ok(result) => Some(result),
            Err(error) => {
                logger
                    .info(&format!("startup_cache_load_failed error={error}"))
                    .map_err(ConfigError::Io)?;
                None
            }
        }
    } else {
        None
    };
    let cache_elapsed_ms = cache_load_started.elapsed().as_millis();
    let (mut state, startup_cache_generated_at_ms) = match cache_result {
        Some(StartupCacheLoadResult::Loaded(cache)) => {
            logger
                .log_timing(
                    "startup_cache_load",
                    &format!(
                        "outcome=loaded elapsed_ms={cache_elapsed_ms} age_ms={} sessions={} panes={} path={}",
                        cache.age_ms,
                        cache.inventory.session_count(),
                        cache.inventory.pane_count(),
                        cache.path.display(),
                    ),
                )
                .map_err(ConfigError::Io)?;
            let mut state = AppState::with_inventory(cache.inventory);
            state.popup_mode = runtime.popup;
            state.system_stats = system_stats;
            state.startup_loading = true;
            state.startup_cache_age_ms = Some(cache.age_ms);
            state.startup_cache_path = Some(cache.path);
            (state, Some(cache.generated_at_ms))
        }
        Some(StartupCacheLoadResult::Missing { path }) => {
            logger
                .log_timing(
                    "startup_cache_load",
                    &format!(
                        "outcome=missing elapsed_ms={cache_elapsed_ms} path={}",
                        path.display()
                    ),
                )
                .map_err(ConfigError::Io)?;
            (
                AppState {
                    popup_mode: runtime.popup,
                    system_stats,
                    startup_loading: true,
                    ..AppState::default()
                },
                None,
            )
        }
        Some(StartupCacheLoadResult::Stale { path, age_ms, .. }) => {
            logger
                .log_timing(
                    "startup_cache_load",
                    &format!(
                        "outcome=stale elapsed_ms={cache_elapsed_ms} age_ms={age_ms} path={}",
                        path.display()
                    ),
                )
                .map_err(ConfigError::Io)?;
            (
                AppState {
                    popup_mode: runtime.popup,
                    system_stats,
                    startup_loading: true,
                    ..AppState::default()
                },
                None,
            )
        }
        Some(StartupCacheLoadResult::Empty { path }) => {
            logger
                .log_timing(
                    "startup_cache_load",
                    &format!(
                        "outcome=empty elapsed_ms={cache_elapsed_ms} path={}",
                        path.display()
                    ),
                )
                .map_err(ConfigError::Io)?;
            (
                AppState {
                    popup_mode: runtime.popup,
                    system_stats,
                    startup_loading: true,
                    ..AppState::default()
                },
                None,
            )
        }
        Some(StartupCacheLoadResult::Invalid { path, reason }) => {
            logger
                .log_timing(
                    "startup_cache_load",
                    &format!(
                        "outcome=invalid elapsed_ms={cache_elapsed_ms} path={} reason={reason}",
                        path.display()
                    ),
                )
                .map_err(ConfigError::Io)?;
            (
                AppState {
                    popup_mode: runtime.popup,
                    system_stats,
                    startup_loading: true,
                    ..AppState::default()
                },
                None,
            )
        }
        None => (
            AppState {
                popup_mode: runtime.popup,
                system_stats,
                startup_loading: true,
                ..AppState::default()
            },
            None,
        ),
    };
    apply_runtime_state_defaults(&mut state, &runtime, &ui_preferences);
    apply_linked_repositories_to_state(&mut state, &runtime);
    apply_ui_preferences_error_to_state(&mut state, &runtime, ui_preferences_error.as_deref());

    let persistent_runtime_diagnostics =
        ui_preferences_error_diagnostics(&runtime, ui_preferences_error.as_deref());

    Ok(PreparedBootstrap {
        runtime,
        logger,
        state,
        claude_native: ClaudeNativeOverlaySummary::default(),
        codex_native: CodexNativeOverlaySummary::default(),
        pi_native: PiNativeOverlaySummary::default(),
        startup_cache_generated_at_ms,
        pending_selection_restore: ui_preferences.selection.clone(),
        persistent_runtime_diagnostics,
    })
}

fn load_and_apply_ui_preferences(
    runtime: &mut RuntimeConfig,
) -> (PersistedUiPreferences, Option<String>) {
    match load_ui_preferences(&runtime.ui_preferences_file) {
        Ok(preferences) => {
            apply_preferences_to_runtime(runtime, &preferences);
            (preferences, None)
        }
        Err(error) => (PersistedUiPreferences::default(), Some(error.to_string())),
    }
}

fn log_ui_preferences_status(
    logger: &mut RunLogger,
    runtime: &RuntimeConfig,
    error: Option<String>,
) -> Result<(), ConfigError> {
    logger
        .info(&format!(
            "ui_preferences path={} default_sort={} theme={}",
            runtime.ui_preferences_file.display(),
            runtime.default_sort.config_label(),
            runtime.theme.label(),
        ))
        .map_err(ConfigError::Io)?;
    if let Some(error) = error {
        logger
            .info(&format!("ui_preferences_load_failed error={error}"))
            .map_err(ConfigError::Io)?;
    }
    Ok(())
}

fn apply_runtime_state_defaults(
    state: &mut AppState,
    runtime: &RuntimeConfig,
    ui_preferences: &PersistedUiPreferences,
) {
    state.popup_mode = runtime.popup;
    state.sort_mode = runtime.default_sort;
    state.notifications.muted = !runtime.notifications_enabled;
    state.notifications.profile = runtime.notification_profile;
    state.notifications.cooldown_ticks = runtime.notification_cooldown_ticks;
    apply_ui_preferences_to_state(state, runtime, ui_preferences);
}

fn apply_linked_repositories_to_state(state: &mut AppState, runtime: &RuntimeConfig) {
    match load_linked_repositories(&runtime.config_file) {
        Ok(links) => {
            let mut skipped = Vec::new();
            let mut linked_repositories = std::collections::BTreeMap::new();
            let mut linked_repository_pane_working_dirs = std::collections::BTreeMap::new();
            for (pane_id, link) in links.links {
                match resolve_git_repository(&link.repository) {
                    Ok(repo_root) => {
                        if let Some(pane) = state.inventory.local_pane(&pane_id) {
                            if link.pane_working_dir != pane.working_dir {
                                skipped
                                    .push(format!("{}: pane identity changed", pane_id.as_str()));
                                continue;
                            }
                        }
                        linked_repository_pane_working_dirs.insert(
                            crate::app::PaneKey::local(pane_id.clone()),
                            link.pane_working_dir,
                        );
                        linked_repositories.insert(pane_id, repo_root);
                    }
                    Err(error) => {
                        skipped.push(format!("{}: {error}", pane_id.as_str()));
                    }
                }
            }
            state.linked_repositories = linked_repositories;
            state.linked_repository_pane_working_dirs = linked_repository_pane_working_dirs;
            state.rebuild_visible_state();
            state.reconcile_selection();
            if !skipped.is_empty() {
                state.operator_alert = Some(OperatorAlert::new(
                    OperatorAlertSource::Diagnostics,
                    OperatorAlertLevel::Warn,
                    format!(
                        "Ignored stale linked repository target(s): {}",
                        skipped.join(", ")
                    ),
                ));
            }
        }
        Err(error) => {
            state.operator_alert = Some(OperatorAlert::new(
                OperatorAlertSource::Diagnostics,
                OperatorAlertLevel::Warn,
                format!("Linked repository state could not be loaded: {error}"),
            ));
        }
    }
}

fn apply_ui_preferences_to_state(
    state: &mut AppState,
    runtime: &RuntimeConfig,
    ui_preferences: &PersistedUiPreferences,
) {
    let mut state_preferences = ui_preferences.clone();
    if runtime.ui_default_sort_configured {
        state_preferences.sort_mode = None;
    }
    state_preferences.apply_to_state(state);
}

fn apply_ui_preferences_error_to_state(
    state: &mut AppState,
    runtime: &RuntimeConfig,
    error: Option<&str>,
) {
    let diagnostics = ui_preferences_error_diagnostics(runtime, error);
    let Some(finding) = diagnostics.first() else {
        return;
    };
    state.operator_alert = Some(OperatorAlert::new(
        OperatorAlertSource::Diagnostics,
        OperatorAlertLevel::Warn,
        finding.summary.clone(),
    ));
    state.runtime_diagnostics.extend(diagnostics);
}

fn ui_preferences_error_diagnostics(
    runtime: &RuntimeConfig,
    error: Option<&str>,
) -> Vec<DoctorFinding> {
    let Some(error) = error else {
        return Vec::new();
    };
    let message = format!(
        "UI preferences at {} could not be loaded: {error}",
        runtime.ui_preferences_file.display()
    );
    vec![DoctorFinding::new(
        "ui-preferences-load-failed",
        DoctorSeverity::Warn,
        DoctorArea::Runtime,
        message,
    )
    .with_next_step("Run foreman --reset-ui-state to remove the persisted UI state file")]
}

fn start_run_logger(runtime: &RuntimeConfig) -> Result<RunLogger, ConfigError> {
    let mut logger = RunLogger::start(
        &runtime.log_dir,
        runtime.log_retention,
        runtime.log_verbosity,
    )
    .map_err(ConfigError::Io)?;
    logger.log_bootstrap(runtime).map_err(ConfigError::Io)?;
    logger
        .debug("bootstrap_debug_logging_enabled")
        .map_err(ConfigError::Io)?;
    Ok(logger)
}

pub(crate) fn bootstrap_state(
    runtime: &RuntimeConfig,
) -> (
    AppState,
    InventoryLoadMetrics,
    BootstrapNativeTiming,
    ClaudeNativeOverlaySummary,
    CodexNativeOverlaySummary,
    PiNativeOverlaySummary,
) {
    let system_stats = SystemStatsService::new(SysinfoSystemStatsBackend::new())
        .snapshot()
        .unwrap_or_default();
    let adapter = TmuxAdapter::new(SystemTmuxBackend::with_target(runtime.tmux_target()));
    match adapter.load_inventory_profiled(runtime.capture_lines) {
        Ok((mut inventory, tmux_metrics)) => {
            let native_started = Instant::now();
            let claude_started = Instant::now();
            let claude_native = apply_configured_claude_signals(
                &mut inventory,
                runtime.claude_native_dir.as_deref(),
                runtime.claude_integration_preference,
            );
            let claude_ms = claude_started.elapsed().as_millis();
            let codex_started = Instant::now();
            let codex_native = apply_configured_codex_signals(
                &mut inventory,
                runtime.codex_native_dir.as_deref(),
                runtime.codex_integration_preference,
            );
            let codex_ms = codex_started.elapsed().as_millis();
            let pi_started = Instant::now();
            let pi_native = apply_configured_pi_signals(
                &mut inventory,
                runtime.pi_native_dir.as_deref(),
                runtime.pi_integration_preference,
            );
            let pi_ms = pi_started.elapsed().as_millis();

            let mut state = AppState {
                popup_mode: runtime.popup,
                system_stats,
                ..AppState::with_inventory(inventory)
            };
            apply_runtime_state_defaults(&mut state, runtime, &PersistedUiPreferences::default());
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
            (
                state,
                tmux_metrics,
                BootstrapNativeTiming {
                    claude_ms,
                    codex_ms,
                    pi_ms,
                    total_ms: native_started.elapsed().as_millis(),
                },
                claude_native,
                codex_native,
                pi_native,
            )
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
                InventoryLoadMetrics::default(),
                BootstrapNativeTiming::default(),
                ClaudeNativeOverlaySummary::default(),
                CodexNativeOverlaySummary::default(),
                PiNativeOverlaySummary::default(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        extension_cards_response_for_pane_with_collector, prepare_runtime_bootstrap, run,
        run_companion_activation, Cli, CompanionServeArgs, RunOutcome,
    };
    use crate::app::{
        inventory, AppState, HarnessKind, OperatorAlertSource, PaneBuilder, PaneId,
        SelectionTarget, SessionBuilder, SortMode, WindowBuilder,
    };
    use crate::config::resolve_paths;
    use crate::services::startup_cache::write_startup_cache;
    use crate::services::ui_preferences::{save_ui_preferences, PersistedUiPreferences};
    use crate::ui::theme::ThemeName;
    use clap::Parser;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn companion_activation_reports_success_and_sets_env() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let marker = temp_dir.path().join("activation.txt");
        let args = CompanionServeArgs {
            bind: "127.0.0.1:0".to_string(),
            token: None,
            source_id: "workstation".to_string(),
            activation_command: Some(format!(
                "printf '%s:%s\\n' \"$FOREMAN_SOURCE_ID\" \"$FOREMAN_PANE_ID\" > {}",
                marker.display()
            )),
            activation_timeout_ms: 10_000,
            allow_send: false,
            all_panes: false,
            max_requests: None,
            ready_file: None,
            json: false,
        };

        let response = run_companion_activation(&args, "%42").expect("activation attempted");

        assert!(response.ok);
        assert_eq!(
            std::fs::read_to_string(marker).unwrap(),
            "workstation:%42\n"
        );
    }

    #[test]
    fn companion_activation_failure_is_reported_as_warning() {
        let args = CompanionServeArgs {
            bind: "127.0.0.1:0".to_string(),
            token: None,
            source_id: "workstation".to_string(),
            activation_command: Some("exit 7".to_string()),
            activation_timeout_ms: 10_000,
            allow_send: false,
            all_panes: false,
            max_requests: None,
            ready_file: None,
            json: false,
        };

        let response = run_companion_activation(&args, "%42").expect("activation attempted");

        assert!(response.attempted);
        assert!(!response.ok);
        assert!(response
            .message
            .expect("failure should include message")
            .contains("status"));
    }

    #[test]
    fn companion_activation_timeout_is_reported_as_warning() {
        let args = CompanionServeArgs {
            bind: "127.0.0.1:0".to_string(),
            token: None,
            source_id: "workstation".to_string(),
            activation_command: Some("sleep 1".to_string()),
            activation_timeout_ms: 25,
            allow_send: false,
            all_panes: false,
            max_requests: None,
            ready_file: None,
            json: false,
        };

        let response = run_companion_activation(&args, "%42").expect("activation attempted");

        assert!(response.attempted);
        assert!(!response.ok);
        assert!(response
            .message
            .expect("timeout should include message")
            .contains("timed out"));
    }

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
    fn run_prints_resolved_config_readout() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let cli = Cli::parse_from([
            "foreman",
            "--config-show",
            "--config-file",
            temp_dir
                .path()
                .join("custom.toml")
                .to_str()
                .expect("utf-8 path"),
            "--log-dir",
            temp_dir.path().join("logs").to_str().expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("run should succeed");

        match outcome {
            RunOutcome::PrintedConfigReadout(readout) => {
                assert!(readout.contains("config_file:"));
                assert!(readout.contains("ui_state:"));
                assert!(readout.contains("startup_cache:"));
                assert!(readout.contains("claude: mode="));
            }
            other => panic!("expected config readout, got {other:?}"),
        }
    }

    #[test]
    fn control_subcommands_reject_top_level_modes() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let cli = Cli::parse_from([
            "foreman",
            "--doctor",
            "--config-file",
            temp_dir
                .path()
                .join("config.toml")
                .to_str()
                .expect("utf-8 path"),
            "agents",
            "--json",
        ]);

        let error = run(cli).expect_err("control command should reject top-level modes");

        assert!(error
            .to_string()
            .contains("control subcommands cannot be combined with --doctor"));
    }

    #[test]
    fn links_help_guides_agents_to_link_working_sessions() {
        let help = Cli::try_parse_from(["foreman", "links", "--help"])
            .expect_err("--help should render help")
            .to_string();

        assert!(help.contains("Repository links let Foreman associate a tmux pane"));
        assert!(help.contains("Use when: an agent is running from notes, Obsidian"));
        assert!(help.contains("echo \"$TMUX_PANE\""));
        assert!(help.contains("foreman agents --json"));
        assert!(
            help.contains("foreman links add --pane %82 --repo ~/git_repositories/foreman --json")
        );
        assert!(help.contains("foreman links list --json"));
        assert!(!help.contains("foreman link --pane"));
        assert!(!help.contains("foreman unlink --pane"));

        let add_help = Cli::try_parse_from(["foreman", "links", "add", "--help"])
            .expect_err("--help should render add help")
            .to_string();
        assert!(add_help.contains("validates that the pane is live"));
        assert!(add_help.contains("`echo $TMUX_PANE`"));
    }

    #[test]
    fn top_level_link_aliases_are_not_supported() {
        let link_error = Cli::try_parse_from([
            "foreman",
            "link",
            "--pane",
            "%82",
            "--repo",
            "/tmp/foreman",
            "--json",
        ])
        .expect_err("top-level link should not parse");
        assert!(link_error
            .to_string()
            .contains("unrecognized subcommand 'link'"));

        let unlink_error = Cli::try_parse_from(["foreman", "unlink", "--pane", "%82", "--json"])
            .expect_err("top-level unlink should not parse");
        assert!(unlink_error
            .to_string()
            .contains("unrecognized subcommand 'unlink'"));
    }

    #[test]
    fn extensions_response_uses_valid_linked_repository_target() {
        let inventory = inventory([SessionBuilder::new("notes").window(
            WindowBuilder::new("main").pane(
                PaneBuilder::agent("%52", HarnessKind::Pi)
                    .working_dir("/tmp/obsidian")
                    .preview("working from notes"),
            ),
        )]);
        let mut state = AppState::with_inventory(inventory);
        state
            .linked_repositories
            .insert(PaneId::new("%52"), PathBuf::from("/tmp/code"));
        state.linked_repository_pane_working_dirs.insert(
            crate::app::PaneKey::local(PaneId::new("%52")),
            Some(PathBuf::from("/tmp/obsidian")),
        );

        let response = extension_cards_response_for_pane_with_collector(
            &state,
            &PaneId::new("%52"),
            |target| {
                assert_eq!(target, &PathBuf::from("/tmp/code"));
                vec![test_extension_card("hk")]
            },
        )
        .expect("extension response");

        assert_eq!(response["paneId"], "%52");
        assert_eq!(response["workspace"], "/tmp/obsidian");
        assert_eq!(response["linkedRepository"], "/tmp/code");
        assert_eq!(response["targetPath"], "/tmp/code");
        assert_eq!(response["extensionCards"][0]["id"], "hk");
    }

    #[test]
    fn extensions_response_ignores_stale_linked_repository_target() {
        let inventory = inventory([SessionBuilder::new("notes")
            .window(WindowBuilder::new("main").pane(
                PaneBuilder::agent("%52", HarnessKind::Pi).working_dir("/tmp/current-notes"),
            ))]);
        let mut state = AppState::with_inventory(inventory);
        state
            .linked_repositories
            .insert(PaneId::new("%52"), PathBuf::from("/tmp/code"));
        state.linked_repository_pane_working_dirs.insert(
            crate::app::PaneKey::local(PaneId::new("%52")),
            Some(PathBuf::from("/tmp/old-notes")),
        );

        let response = extension_cards_response_for_pane_with_collector(
            &state,
            &PaneId::new("%52"),
            |target| {
                assert_eq!(target, &PathBuf::from("/tmp/current-notes"));
                Vec::new()
            },
        )
        .expect("extension response");

        assert_eq!(response["linkedRepository"], serde_json::Value::Null);
        assert_eq!(response["targetPath"], "/tmp/current-notes");
    }

    #[test]
    fn extensions_response_rejects_unknown_pane() {
        let state = AppState::with_inventory(inventory([SessionBuilder::new("empty")]));

        let error = extension_cards_response_for_pane_with_collector(
            &state,
            &PaneId::new("%999999"),
            |_| Vec::new(),
        )
        .expect_err("unknown pane should fail");

        assert!(error
            .to_string()
            .contains("cannot inspect extensions for unknown pane %999999"));
    }

    fn test_extension_card(id: &str) -> crate::services::extensions::ControlExtensionCard {
        crate::services::extensions::ControlExtensionCard {
            id: id.to_string(),
            title: "Harness Kit".to_string(),
            status: "ready".to_string(),
            status_label: "READY".to_string(),
            summary: "ready".to_string(),
            rows: Vec::new(),
            actions: Vec::new(),
        }
    }

    #[test]
    fn links_list_prints_persisted_links_as_json() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config = temp_dir.path().join("config.toml");
        let mut links = crate::services::linked_repositories::LinkedRepositories::default();
        links.insert(
            PaneId::new("%82"),
            PathBuf::from("/tmp/foreman"),
            Some(PathBuf::from("/tmp/obsidian-vault")),
        );
        crate::services::linked_repositories::save_linked_repositories(&config, &links)
            .expect("save links");
        let cli = Cli::parse_from([
            "foreman",
            "--config-file",
            config.to_str().expect("utf-8 path"),
            "links",
            "list",
            "--json",
        ]);

        let outcome = run(cli).expect("run should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected links list JSON, got {outcome:?}");
        };
        assert_eq!(value["action"], "links.list");
        assert_eq!(value["links"][0]["paneId"], "%82");
        assert_eq!(value["links"][0]["linkedRepository"], "/tmp/foreman");
        assert_eq!(value["links"][0]["paneWorkingDir"], "/tmp/obsidian-vault");
        assert_eq!(
            value["path"],
            temp_dir
                .path()
                .join("linked-repositories.json")
                .to_string_lossy()
                .as_ref()
        );
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
    fn runtime_bootstrap_uses_fresh_popup_startup_cache() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let cli = Cli::parse_from([
            "foreman",
            "--popup",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        std::fs::write(
            &config_file,
            r#"
[ui]
default_sort = "attention-recent"
"#,
        )
        .expect("config should be writable");
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        let runtime = crate::config::RuntimeConfig::from_sources(
            paths.clone(),
            crate::config::AppConfig::default(),
            &cli,
        );
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .preview("Claude Code ready"),
            ),
        )]);
        write_startup_cache(&runtime, &inventory).expect("cache write should succeed");
        save_ui_preferences(
            &paths.ui_preferences_file,
            &PersistedUiPreferences {
                selection: Some(SelectionTarget::Pane("alpha:claude".into())),
                ..PersistedUiPreferences::default()
            },
        )
        .expect("ui preferences should save");

        let prepared =
            prepare_runtime_bootstrap(&cli, paths).expect("runtime bootstrap should succeed");

        assert!(prepared.state.startup_loading);
        assert!(prepared.state.startup_cache_age_ms.is_some());
        assert_eq!(prepared.state.inventory, inventory);
        assert_eq!(prepared.state.sort_mode, SortMode::AttentionFirst);
        assert_eq!(
            prepared.state.selection,
            Some(SelectionTarget::Pane("alpha:claude".into()))
        );
        assert!(prepared.startup_cache_generated_at_ms.is_some());
    }

    #[test]
    fn runtime_bootstrap_skips_startup_cache_for_explicit_source_scope() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let cli = Cli::parse_from([
            "foreman",
            "--popup",
            "--source",
            "coder",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        std::fs::write(&config_file, "[sources]\ndefault_scope = \"all\"\n")
            .expect("config should be writable");
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        let runtime = crate::config::RuntimeConfig::from_sources(
            paths.clone(),
            crate::config::AppConfig::default(),
            &cli,
        );
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha"),
            ),
        )]);
        write_startup_cache(&runtime, &inventory).expect("cache write should succeed");

        let prepared =
            prepare_runtime_bootstrap(&cli, paths).expect("runtime bootstrap should succeed");

        assert!(prepared.state.inventory.sessions.is_empty());
        assert!(prepared.state.startup_cache_age_ms.is_none());
        assert!(prepared.startup_cache_generated_at_ms.is_none());
    }

    #[test]
    fn generated_default_config_allows_persisted_theme_and_sort() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        crate::config::write_default_config(&config_file).expect("config should be written");
        let cli = Cli::parse_from([
            "foreman",
            "--popup",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        save_ui_preferences(
            &paths.ui_preferences_file,
            &PersistedUiPreferences {
                sort_mode: Some(SortMode::AttentionFirst),
                theme: Some(ThemeName::Dracula),
                ..PersistedUiPreferences::default()
            },
        )
        .expect("ui preferences should save");

        let prepared = prepare_runtime_bootstrap(&cli, paths).expect("bootstrap should succeed");

        assert_eq!(prepared.runtime.theme, ThemeName::Dracula);
        assert_eq!(prepared.state.sort_mode, SortMode::AttentionFirst);
    }

    #[test]
    fn invalid_ui_state_adds_persistent_runtime_diagnostic() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let cli = Cli::parse_from([
            "foreman",
            "--popup",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        std::fs::create_dir_all(paths.ui_preferences_file.parent().expect("parent exists"))
            .expect("ui state parent should exist");
        std::fs::write(&paths.ui_preferences_file, "{invalid-json")
            .expect("ui state should be writable");

        let prepared = prepare_runtime_bootstrap(&cli, paths).expect("bootstrap should succeed");

        assert!(prepared
            .persistent_runtime_diagnostics
            .iter()
            .any(|finding| finding.id == "ui-preferences-load-failed"));
        assert!(prepared
            .state
            .runtime_diagnostics
            .iter()
            .any(|finding| finding.id == "ui-preferences-load-failed"));
    }

    #[test]
    fn explicit_config_ui_keys_win_over_persisted_preferences() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        std::fs::write(
            &config_file,
            r#"
[ui]
theme = "catppuccin"
default_sort = "stable"
"#,
        )
        .expect("config should be writable");
        let cli = Cli::parse_from([
            "foreman",
            "--popup",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        save_ui_preferences(
            &paths.ui_preferences_file,
            &PersistedUiPreferences {
                sort_mode: Some(SortMode::AttentionFirst),
                theme: Some(ThemeName::Dracula),
                ..PersistedUiPreferences::default()
            },
        )
        .expect("ui preferences should save");

        let prepared = prepare_runtime_bootstrap(&cli, paths).expect("bootstrap should succeed");

        assert_eq!(prepared.runtime.theme, ThemeName::Catppuccin);
        assert_eq!(prepared.state.sort_mode, SortMode::Stable);
    }

    #[test]
    fn reset_ui_state_removes_persisted_preferences_and_exits() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let paths =
            resolve_paths(Some(&config_file), Some(&log_dir)).expect("paths should resolve");
        save_ui_preferences(
            &paths.ui_preferences_file,
            &PersistedUiPreferences::default(),
        )
        .expect("ui preferences should save");
        let cli = Cli::parse_from([
            "foreman",
            "--reset-ui-state",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
        ]);

        let outcome = run(cli).expect("run should succeed");

        match outcome {
            RunOutcome::ResetUiState { path, removed } => {
                assert_eq!(path, paths.ui_preferences_file);
                assert!(removed);
                assert!(!path.exists());
            }
            other => panic!("expected reset outcome, got {other:?}"),
        }
    }

    #[test]
    fn sources_add_ssh_persists_and_lists_source() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let add = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "add",
            "ssh",
            "coder",
            "--host",
            "coder.example",
            "--foreman",
            "/bin/foreman",
            "--tmux-server-name",
            "user",
            "--label",
            "Coder",
            "--json",
        ]);
        run(add).expect("add source should succeed");

        let list = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "list",
            "--json",
        ]);
        let outcome = run(list).expect("list sources should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected JSON outcome, got {outcome:?}");
        };
        let sources = value["sources"].as_array().expect("sources array");
        assert!(sources.iter().any(|source| source["id"] == "local"));
        assert!(sources.iter().any(|source| source["id"] == "coder"));
        let config = std::fs::read_to_string(config_file).expect("config should be readable");
        assert!(config.contains("[sources.coder]"));
        assert!(config.contains("tmux_server_name = \"user\""));
    }

    #[test]
    fn agents_sources_all_merges_snapshot_response_with_source_identity() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let snapshot_file = temp_dir.path().join("mac-source.agents.json");
        std::fs::write(
            &snapshot_file,
            r#"{
  "schemaVersion": 1,
  "sourceId": "mac",
  "capturedAtUnixMs": 9999999990000,
  "expiresAtUnixMs": 9999999999999,
  "agentsResponseSchemaVersion": 2,
  "response": {
    "schemaVersion": 2,
    "generatedAtUnixMs": 9999999990000,
    "inventory": {"totalSessions":1,"totalWindows":1,"totalPanes":1,"visibleSessions":1,"visibleWindows":1,"visiblePanes":1},
    "entries": [{"id":"pane:%77","paneId":"%77","sessionId":"$0","sessionName":"mac","windowId":"@0","windowName":"mac","title":"mac","navigationTitle":"mac","harness":null,"harnessLabel":null,"status":"unknown","statusLabel":"UNKNOWN","statusSource":null,"integrationMode":null,"isAgent":false,"currentCommand":null,"runtimeCommand":null,"workingDir":null,"linkedRepository":null,"workspaceName":null,"preview":"","previewProvenance":"captured","activityScore":0,"statusRank":4,"lastActivityUnixMs":null,"lastStatusChangeUnixMs":null,"activeRunCount":null,"pullRequest":null}],
    "diagnostics": []
  },
  "health": {"status":"ok","message":null}
}
"#,
        )
        .expect("snapshot should be writable");
        std::fs::write(
            &config_file,
            format!(
                "[sources]\ndefault_scope = \"all\"\n\n[sources.mac]\nkind = \"snapshot\"\nlabel = \"Mac\"\npath = \"{}\"\n",
                snapshot_file.display()
            ),
        )
        .expect("config should be writable");
        let cli = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "agents",
            "--json",
            "--all-panes",
        ]);
        let outcome = run(cli).expect("agents should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected JSON outcome, got {outcome:?}");
        };
        let entries = value["entries"].as_array().expect("entries array");
        assert!(entries.iter().any(|entry| {
            entry["sourceId"] == "mac"
                && entry["sourcePaneId"] == "source:mac:pane:%77"
                && entry["paneId"] == "%77"
        }));
        assert_eq!(value["partialFailureCount"], 0);
    }

    #[test]
    fn sources_add_companion_persists_and_lists_source() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let add = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "add",
            "companion",
            "mac",
            "--endpoint",
            "127.0.0.1:4040",
            "--label",
            "Mac",
            "--allow-send",
            "--activation-command",
            "echo activate",
            "--json",
        ]);
        run(add).expect("add companion source should succeed");

        let list = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "list",
            "--json",
        ]);
        let outcome = run(list).expect("list sources should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected JSON outcome, got {outcome:?}");
        };
        let sources = value["sources"].as_array().expect("sources array");
        assert!(sources.iter().any(|source| {
            source["id"] == "mac" && source["kind"] == "companion" && source["label"] == "Mac"
        }));
        let config = std::fs::read_to_string(config_file).expect("config should be readable");
        assert!(config.contains("kind = \"companion\""));
        assert!(config.contains("allow_send = true"));
        assert!(config.contains("activation_command = \"echo activate\""));
    }

    #[test]
    fn sources_add_snapshot_persists_and_lists_source() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let snapshot_file = temp_dir.path().join("mac-source.agents.json");
        let log_dir = temp_dir.path().join("logs");
        let add = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "add",
            "snapshot",
            "mac",
            "--path",
            snapshot_file.to_str().expect("utf-8 path"),
            "--label",
            "Mac",
            "--json",
        ]);
        run(add).expect("add snapshot source should succeed");

        let list = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "sources",
            "list",
            "--json",
        ]);
        let outcome = run(list).expect("list sources should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected JSON outcome, got {outcome:?}");
        };
        let sources = value["sources"].as_array().expect("sources array");
        assert!(sources.iter().any(|source| {
            source["id"] == "mac" && source["kind"] == "snapshot" && source["label"] == "Mac"
        }));
        let config = std::fs::read_to_string(config_file).expect("config should be readable");
        assert!(config.contains("[sources.mac]"));
        assert!(config.contains("kind = \"snapshot\""));
    }

    #[test]
    fn agents_sources_all_merges_fake_ssh_response_with_source_identity() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_file = temp_dir.path().join("config.toml");
        let log_dir = temp_dir.path().join("logs");
        let fake_ssh = temp_dir.path().join("ssh");
        std::fs::write(
            &fake_ssh,
            "#!/bin/sh\ncat <<'JSON'\n{\"schemaVersion\":2,\"generatedAtUnixMs\":1,\"inventory\":{\"totalSessions\":1,\"totalWindows\":1,\"totalPanes\":1,\"visibleSessions\":1,\"visibleWindows\":1,\"visiblePanes\":1},\"entries\":[{\"id\":\"pane:%42\",\"paneId\":\"%42\",\"sessionId\":\"$0\",\"sessionName\":\"remote\",\"windowId\":\"@0\",\"windowName\":\"remote\",\"title\":\"remote\",\"navigationTitle\":\"remote\",\"harness\":null,\"harnessLabel\":null,\"status\":\"unknown\",\"statusLabel\":\"UNKNOWN\",\"statusSource\":null,\"integrationMode\":null,\"isAgent\":false,\"currentCommand\":null,\"runtimeCommand\":null,\"workingDir\":null,\"linkedRepository\":null,\"workspaceName\":null,\"preview\":\"\",\"previewProvenance\":\"captured\",\"activityScore\":0,\"statusRank\":4,\"lastActivityUnixMs\":null,\"lastStatusChangeUnixMs\":null,\"activeRunCount\":null,\"pullRequest\":null}],\"diagnostics\":[]}\nJSON\n",
        )
        .expect("fake ssh should be writable");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&fake_ssh).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&fake_ssh, perms).unwrap();
        }
        std::fs::write(
            &config_file,
            format!(
                "[sources]\ndefault_scope = \"all\"\n\n[sources.coder]\nkind = \"ssh\"\nlabel = \"Coder\"\nhost = \"coder.example\"\nforeman = \"/bin/foreman\"\nssh = \"{}\"\n",
                fake_ssh.display()
            ),
        )
        .expect("config should be writable");
        let cli = Cli::parse_from([
            "foreman",
            "--config-file",
            config_file.to_str().expect("utf-8 path"),
            "--log-dir",
            log_dir.to_str().expect("utf-8 path"),
            "agents",
            "--json",
            "--all-panes",
        ]);
        let outcome = run(cli).expect("agents should succeed");

        let RunOutcome::ControlJson(value) = outcome else {
            panic!("expected JSON outcome, got {outcome:?}");
        };
        let entries = value["entries"].as_array().expect("entries array");
        assert!(entries.iter().any(|entry| {
            entry["sourceId"] == "coder"
                && entry["sourcePaneId"] == "source:coder:pane:%42"
                && entry["paneId"] == "%42"
        }));
        assert_eq!(value["partialFailureCount"], 0);
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
