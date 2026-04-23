use crate::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use crate::app::{HarnessKind, IntegrationMode, Inventory, Pane};
use crate::config::{
    default_claude_native_dir, default_codex_native_dir, default_pi_native_dir,
    write_default_config, RuntimeConfig,
};
use crate::integrations::{
    apply_configured_claude_signals, apply_configured_codex_signals, apply_configured_pi_signals,
    ClaudeNativeOverlaySummary, CodexNativeOverlaySummary, PiNativeOverlaySummary,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIN_CODEX_HOOK_VERSION: (u64, u64, u64) = (0, 116, 0);
const HOOK_ERROR_SIGNATURES: &[&str] = &[
    "hook error",
    "pretooluse:bash hook error",
    "foreman-claude-hook failed",
    "foreman-codex-hook failed",
    "foreman-pi-hook failed",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorSeverity {
    Ok,
    Info,
    Warn,
    Error,
}

impl DoctorSeverity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorArea {
    Machine,
    Config,
    Repo,
    Runtime,
}

impl DoctorArea {
    fn label(self, repo_path: Option<&Path>) -> String {
        match self {
            Self::Machine => "Machine".to_string(),
            Self::Config => "Config".to_string(),
            Self::Repo => repo_path
                .map(|path| format!("Repo: {}", path.display()))
                .unwrap_or_else(|| "Repo".to_string()),
            Self::Runtime => "Runtime hints".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorFinding {
    pub id: String,
    pub severity: DoctorSeverity,
    pub area: DoctorArea,
    pub provider: Option<HarnessKind>,
    pub pane_id: Option<String>,
    pub repo_path: Option<PathBuf>,
    pub summary: String,
    pub detail: Option<String>,
    pub evidence: Vec<String>,
    pub next_step: Option<String>,
}

impl DoctorFinding {
    pub fn new(
        id: impl Into<String>,
        severity: DoctorSeverity,
        area: DoctorArea,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            area,
            provider: None,
            pane_id: None,
            repo_path: None,
            summary: summary.into(),
            detail: None,
            evidence: Vec::new(),
            next_step: None,
        }
    }

    pub fn with_provider(mut self, provider: HarnessKind) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn with_pane_id(mut self, pane_id: impl Into<String>) -> Self {
        self.pane_id = Some(pane_id.into());
        self
    }

    pub fn with_repo_path(mut self, repo_path: impl Into<PathBuf>) -> Self {
        self.repo_path = Some(repo_path.into());
        self
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn with_next_step(mut self, next_step: impl Into<String>) -> Self {
        self.next_step = Some(next_step.into());
        self
    }

    pub fn push_evidence(&mut self, evidence: impl Into<String>) {
        self.evidence.push(evidence.into());
    }

    pub fn matches_context(
        &self,
        provider: Option<HarnessKind>,
        pane_id: Option<&str>,
        workspace_path: Option<&Path>,
    ) -> bool {
        if let Some(finding_pane_id) = self.pane_id.as_deref() {
            return pane_id == Some(finding_pane_id);
        }

        if let Some(finding_provider) = self.provider {
            if Some(finding_provider) != provider {
                return false;
            }
        }

        if let Some(repo_path) = self.repo_path.as_deref() {
            if let Some(workspace_path) = workspace_path {
                return workspace_path.starts_with(repo_path)
                    || repo_path.starts_with(workspace_path);
            }

            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorFixStatus {
    Planned,
    Written,
    Unchanged,
    Skipped,
}

impl DoctorFixStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "PLANNED",
            Self::Written => "WRITTEN",
            Self::Unchanged => "UNCHANGED",
            Self::Skipped => "SKIPPED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorFixResult {
    pub provider: Option<HarnessKind>,
    pub path: PathBuf,
    pub status: DoctorFixStatus,
    pub message: String,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorReport {
    pub repo_path: Option<PathBuf>,
    pub findings: Vec<DoctorFinding>,
    pub fixes: Vec<DoctorFixResult>,
}

impl DoctorReport {
    pub fn blocking_findings(&self) -> impl Iterator<Item = &DoctorFinding> {
        self.findings
            .iter()
            .filter(|finding| finding.severity == DoctorSeverity::Error)
    }

    pub fn has_blocking_findings(&self) -> bool {
        self.blocking_findings().next().is_some()
    }

    pub fn to_setup_text(
        &self,
        dry_run: bool,
        setup_command: &str,
        scopes: SetupScopeSelection,
        provider_summary: &str,
    ) -> String {
        let mut output = String::new();
        output.push_str(if dry_run {
            "Foreman setup preview\n"
        } else {
            "Foreman setup\n"
        });
        let repo_line = match self.repo_path.as_deref() {
            Some(path) => format!("Repo: {}\n", path.display()),
            None if scopes.user && scopes.project => {
                "Repo: none detected from the current directory; user-scoped setup can be applied and project-scoped changes will be skipped.\n".to_string()
            }
            None if scopes.user => {
                "Repo: none detected from the current directory; applying user-scoped setup.\n"
                    .to_string()
            }
            None => {
                "Repo: none detected from the current directory; project-scoped changes will be skipped.\n"
                    .to_string()
            }
        };
        output.push_str(&repo_line);
        output.push_str(&format!("Targets: {}\n", scopes.summary()));
        output.push_str(&format!("Providers: {provider_summary}\n"));
        output.push('\n');

        if self.fixes.is_empty() {
            output.push_str("No safe setup changes were needed.\n");
        } else {
            output.push_str(if dry_run {
                "Planned changes\n"
            } else {
                "Applied changes\n"
            });
            for fix in &self.fixes {
                output.push_str(&format!(
                    "  {:<8} {} {}\n",
                    fix.status.label(),
                    fix.path.display(),
                    fix.message
                ));
                if let Some(preview) = &fix.preview {
                    for line in preview.lines() {
                        output.push_str(&format!("          {}\n", line));
                    }
                }
            }
        }

        let remaining = self
            .findings
            .iter()
            .filter(|finding| {
                matches!(
                    finding.severity,
                    DoctorSeverity::Warn | DoctorSeverity::Error
                )
            })
            .collect::<Vec<_>>();
        if !remaining.is_empty() {
            output.push_str("\nStill needs attention\n");
            for finding in remaining.into_iter().take(6) {
                output.push_str(&format!(
                    "  {:<8} {}\n",
                    finding.severity.label(),
                    finding.summary
                ));
                if let Some(next_step) = &finding.next_step {
                    output.push_str(&format!("          next: {}\n", next_step));
                }
            }
        }

        output.push_str("\nNext\n");
        if dry_run {
            output.push_str(&format!("  {setup_command}\n"));
            output.push_str(&format!(
                "  {}\n",
                doctor_command(self.repo_path.as_deref())
            ));
        } else {
            output.push_str(&format!(
                "  {}\n",
                doctor_command(self.repo_path.as_deref())
            ));
            output.push_str("  foreman\n");
        }

        output.trim_end().to_string()
    }

    pub fn to_text(&self) -> String {
        let mut output = String::new();

        for area in [
            DoctorArea::Machine,
            DoctorArea::Config,
            DoctorArea::Repo,
            DoctorArea::Runtime,
        ] {
            let area_findings = self
                .findings
                .iter()
                .filter(|finding| finding.area == area)
                .collect::<Vec<_>>();
            if area_findings.is_empty() {
                continue;
            }

            output.push_str(&area.label(self.repo_path.as_deref()));
            output.push('\n');
            for finding in area_findings {
                output.push_str(&format!(
                    "  {:<8} {}\n",
                    finding.severity.label(),
                    finding.summary
                ));
                if let Some(detail) = &finding.detail {
                    output.push_str(&format!("          {}\n", detail));
                }
                for evidence in &finding.evidence {
                    output.push_str(&format!("          {}\n", evidence));
                }
                if let Some(next_step) = &finding.next_step {
                    output.push_str(&format!("          next: {}\n", next_step));
                }
            }
            output.push('\n');
        }

        if !self.fixes.is_empty() {
            output.push_str("Fixes\n");
            for fix in &self.fixes {
                output.push_str(&format!(
                    "  {:<8} {} {}\n",
                    fix.status.label(),
                    fix.path.display(),
                    fix.message
                ));
                if let Some(preview) = &fix.preview {
                    for line in preview.lines() {
                        output.push_str(&format!("          {}\n", line));
                    }
                }
            }
        }

        output.trim_end().to_string()
    }
}

pub fn primary_runtime_alert_finding(findings: &[DoctorFinding]) -> Option<&DoctorFinding> {
    findings
        .iter()
        .filter(|finding| {
            finding.area == DoctorArea::Runtime
                && matches!(
                    finding.severity,
                    DoctorSeverity::Warn | DoctorSeverity::Error
                )
        })
        .max_by(|left, right| {
            left.severity
                .cmp(&right.severity)
                .then_with(|| left.summary.cmp(&right.summary))
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorFixMode {
    ReportOnly,
    DryRun,
    Apply,
}

impl DoctorFixMode {
    fn wants_fixes(self) -> bool {
        !matches!(self, Self::ReportOnly)
    }

    fn writes(self) -> bool {
        matches!(self, Self::Apply)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetupScopeSelection {
    pub user: bool,
    pub project: bool,
}

impl SetupScopeSelection {
    pub(crate) fn resolved(self, repo_available: bool) -> Self {
        if self.user || self.project {
            return self;
        }

        if repo_available {
            Self {
                user: false,
                project: true,
            }
        } else {
            Self {
                user: true,
                project: false,
            }
        }
    }

    pub(crate) fn summary(self) -> String {
        match (self.user, self.project) {
            (true, true) => "user + project".to_string(),
            (true, false) => "user".to_string(),
            (false, true) => "project".to_string(),
            (false, false) => "none".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SetupProviderSelection {
    pub claude: bool,
    pub codex: bool,
    pub pi: bool,
}

impl SetupProviderSelection {
    fn includes(self, provider: HarnessKind) -> bool {
        let selection = if self.claude || self.codex || self.pi {
            self
        } else {
            Self {
                claude: true,
                codex: true,
                pi: true,
            }
        };

        match provider {
            HarnessKind::ClaudeCode => selection.claude,
            HarnessKind::CodexCli => selection.codex,
            HarnessKind::Pi => selection.pi,
            HarnessKind::GeminiCli | HarnessKind::OpenCode => false,
        }
    }

    pub(crate) fn has_explicit_selection(self) -> bool {
        self.claude || self.codex || self.pi
    }

    pub(crate) fn selected_flags(self) -> Vec<&'static str> {
        let mut flags = Vec::new();
        if self.claude {
            flags.push("--claude");
        }
        if self.codex {
            flags.push("--codex");
        }
        if self.pi {
            flags.push("--pi");
        }
        flags
    }

    pub(crate) fn selected_labels(self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        let selection = if self.has_explicit_selection() {
            self
        } else {
            Self {
                claude: true,
                codex: true,
                pi: true,
            }
        };
        if selection.claude {
            labels.push(HarnessKind::ClaudeCode.label());
        }
        if selection.codex {
            labels.push(HarnessKind::CodexCli.label());
        }
        if selection.pi {
            labels.push(HarnessKind::Pi.label());
        }
        labels
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOptions {
    pub repo_path: Option<PathBuf>,
    pub fix_mode: DoctorFixMode,
    pub setup_scopes: SetupScopeSelection,
    pub setup_providers: SetupProviderSelection,
    pub config_parse_error: Option<String>,
}

impl Default for DoctorOptions {
    fn default() -> Self {
        Self {
            repo_path: None,
            fix_mode: DoctorFixMode::ReportOnly,
            setup_scopes: SetupScopeSelection::default(),
            setup_providers: SetupProviderSelection::default(),
            config_parse_error: None,
        }
    }
}

#[derive(Debug)]
pub enum DoctorError {
    Config(crate::config::ConfigError),
    Io(std::io::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for DoctorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Serialize(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DoctorError {}

impl From<crate::config::ConfigError> for DoctorError {
    fn from(error: crate::config::ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<std::io::Error> for DoctorError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for DoctorError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeDoctorContext<'a> {
    pub runtime: &'a RuntimeConfig,
    pub config_exists: bool,
    pub inventory: &'a Inventory,
    pub claude_native: &'a ClaudeNativeOverlaySummary,
    pub codex_native: &'a CodexNativeOverlaySummary,
    pub pi_native: &'a PiNativeOverlaySummary,
}

#[derive(Debug, Clone)]
struct LiveRuntimeSnapshot {
    inventory: Inventory,
    claude_native: ClaudeNativeOverlaySummary,
    codex_native: CodexNativeOverlaySummary,
    pi_native: PiNativeOverlaySummary,
}

pub fn collect_report(
    runtime: &RuntimeConfig,
    options: &DoctorOptions,
) -> Result<DoctorReport, DoctorError> {
    let repo_path = resolve_requested_repo_root(options.repo_path.as_deref());
    let mut findings = Vec::new();
    let provider_scope = options.setup_providers;

    findings.extend(filter_provider_findings(
        machine_findings(runtime),
        provider_scope,
    ));
    findings.extend(filter_provider_findings(
        config_findings(runtime, options.config_parse_error.as_deref()),
        provider_scope,
    ));

    if let Some(repo_path) = repo_path.as_deref() {
        findings.extend(filter_provider_findings(
            repo_findings(repo_path),
            provider_scope,
        ));
    }

    let live_runtime = match live_runtime_context(runtime) {
        Ok(Some(snapshot)) => {
            let context = RuntimeDoctorContext {
                runtime,
                config_exists: runtime.config_file.exists(),
                inventory: &snapshot.inventory,
                claude_native: &snapshot.claude_native,
                codex_native: &snapshot.codex_native,
                pi_native: &snapshot.pi_native,
            };
            findings.extend(runtime_findings_scoped(
                &context,
                repo_path.as_deref(),
                provider_scope,
            ));
            true
        }
        Ok(None) => false,
        Err(message) => {
            findings.push(
                DoctorFinding::new(
                    "runtime-live-unavailable",
                    DoctorSeverity::Info,
                    DoctorArea::Runtime,
                    "Live tmux inventory was not available for runtime diagnosis.",
                )
                .with_detail(message),
            );
            false
        }
    };

    if !live_runtime && repo_path.is_none() {
        findings.extend(latest_log_runtime_findings(runtime));
    }

    let mut fixes = Vec::new();
    if options.fix_mode.wants_fixes() {
        fixes.extend(apply_safe_fixes(
            runtime,
            repo_path.as_deref(),
            options.fix_mode,
            options.setup_scopes,
            options.setup_providers,
        )?);
        findings = Vec::new();
        findings.extend(filter_provider_findings(
            machine_findings(runtime),
            provider_scope,
        ));
        findings.extend(filter_provider_findings(
            config_findings(runtime, options.config_parse_error.as_deref()),
            provider_scope,
        ));
        if let Some(repo_path) = repo_path.as_deref() {
            findings.extend(filter_provider_findings(
                repo_findings(repo_path),
                provider_scope,
            ));
        }
        if let Ok(Some(snapshot)) = live_runtime_context(runtime) {
            let context = RuntimeDoctorContext {
                runtime,
                config_exists: runtime.config_file.exists(),
                inventory: &snapshot.inventory,
                claude_native: &snapshot.claude_native,
                codex_native: &snapshot.codex_native,
                pi_native: &snapshot.pi_native,
            };
            findings.extend(runtime_findings_scoped(
                &context,
                repo_path.as_deref(),
                provider_scope,
            ));
        } else if repo_path.is_none() {
            findings.extend(latest_log_runtime_findings(runtime));
        }
    }

    sort_findings(&mut findings);

    Ok(DoctorReport {
        repo_path,
        findings,
        fixes,
    })
}

pub fn runtime_findings(context: &RuntimeDoctorContext<'_>) -> Vec<DoctorFinding> {
    runtime_findings_scoped(context, None, SetupProviderSelection::default())
}

fn runtime_findings_scoped(
    context: &RuntimeDoctorContext<'_>,
    repo_scope: Option<&Path>,
    provider_scope: SetupProviderSelection,
) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();

    if !context.config_exists {
        findings.push(
            DoctorFinding::new(
                "config-missing-runtime",
                DoctorSeverity::Warn,
                DoctorArea::Runtime,
                format!(
                    "Foreman is running with defaults because {} does not exist.",
                    context.runtime.config_file.display()
                ),
            )
            .with_next_step(format!("Run {}", setup_command(repo_scope))),
        );
    }

    for (provider, warnings) in [
        (
            HarnessKind::ClaudeCode,
            context.claude_native.warnings.as_slice(),
        ),
        (
            HarnessKind::CodexCli,
            context.codex_native.warnings.as_slice(),
        ),
        (HarnessKind::Pi, context.pi_native.warnings.as_slice()),
    ] {
        if provider_scope.includes(provider) {
            findings.extend(provider_runtime_findings(
                context.runtime,
                context.inventory,
                provider,
                warnings,
                repo_scope,
            ));
        }
    }

    sort_findings(&mut findings);
    findings
}

fn machine_findings(runtime: &RuntimeConfig) -> Vec<DoctorFinding> {
    let mut findings = vec![
        command_check("tmux", &["-V"], None, DoctorArea::Machine),
        command_check(
            "foreman-claude-hook",
            &["--help"],
            Some(HarnessKind::ClaudeCode),
            DoctorArea::Machine,
        ),
        command_check(
            "foreman-codex-hook",
            &["--help"],
            Some(HarnessKind::CodexCli),
            DoctorArea::Machine,
        ),
        command_check(
            "foreman-pi-hook",
            &["--help"],
            Some(HarnessKind::Pi),
            DoctorArea::Machine,
        ),
        command_check(
            "claude",
            &["--version"],
            Some(HarnessKind::ClaudeCode),
            DoctorArea::Machine,
        ),
        command_check(
            "codex",
            &["--version"],
            Some(HarnessKind::CodexCli),
            DoctorArea::Machine,
        ),
        command_check(
            "pi",
            &["--version"],
            Some(HarnessKind::Pi),
            DoctorArea::Machine,
        ),
    ];

    findings.extend(codex_version_findings());

    findings.push(
        DoctorFinding::new(
            "log-dir",
            if runtime.log_dir.exists() {
                DoctorSeverity::Ok
            } else {
                DoctorSeverity::Info
            },
            DoctorArea::Machine,
            if runtime.log_dir.exists() {
                format!(
                    "Log directory is available at {}",
                    runtime.log_dir.display()
                )
            } else {
                format!(
                    "Log directory has not been created yet at {}",
                    runtime.log_dir.display()
                )
            },
        )
        .with_detail(format!(
            "Claude native dir defaults to {}, Codex to {}, Pi to {}.",
            default_claude_native_dir(&runtime.log_dir).display(),
            default_codex_native_dir(&runtime.log_dir).display(),
            default_pi_native_dir(&runtime.log_dir).display(),
        )),
    );

    findings
}

fn filter_provider_findings(
    findings: Vec<DoctorFinding>,
    provider_scope: SetupProviderSelection,
) -> Vec<DoctorFinding> {
    findings
        .into_iter()
        .filter(|finding| {
            finding
                .provider
                .is_none_or(|provider| provider_scope.includes(provider))
        })
        .collect()
}

fn config_findings(
    runtime: &RuntimeConfig,
    config_parse_error: Option<&str>,
) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    findings.push(
        DoctorFinding::new(
            "config-file",
            if config_parse_error.is_some() {
                DoctorSeverity::Error
            } else if runtime.config_file.exists() {
                DoctorSeverity::Ok
            } else {
                DoctorSeverity::Warn
            },
            DoctorArea::Config,
            if let Some(_parse_error) = config_parse_error {
                format!(
                    "Foreman config exists at {} but could not be parsed",
                    runtime.config_file.display()
                )
            } else if runtime.config_file.exists() {
                format!(
                    "Foreman config is initialized at {}",
                    runtime.config_file.display()
                )
            } else {
                format!(
                    "Foreman config is not initialized at {}",
                    runtime.config_file.display()
                )
            },
        )
        .with_next_step(format!("Run {}", setup_command(None))),
    );
    if let Some(parse_error) = config_parse_error {
        if let Some(finding) = findings.last_mut() {
            finding.detail = Some(parse_error.to_string());
        }
    }

    if config_parse_error.is_some() {
        return findings;
    }

    for (provider, preference, native_dir) in [
        (
            HarnessKind::ClaudeCode,
            context_preference_label(runtime.claude_integration_preference),
            runtime.claude_native_dir.as_deref(),
        ),
        (
            HarnessKind::CodexCli,
            context_preference_label(runtime.codex_integration_preference),
            runtime.codex_native_dir.as_deref(),
        ),
        (
            HarnessKind::Pi,
            context_preference_label(runtime.pi_integration_preference),
            runtime.pi_native_dir.as_deref(),
        ),
    ] {
        let mut finding = DoctorFinding::new(
            format!("{}-integration-config", provider.filter_label()),
            DoctorSeverity::Info,
            DoctorArea::Config,
            format!(
                "{} integration preference is {}.",
                provider.label(),
                preference
            ),
        )
        .with_provider(provider);
        if let Some(native_dir) = native_dir {
            finding.push_evidence(format!("native_dir={}", native_dir.display()));
        }
        findings.push(finding);
    }

    findings
}

fn repo_findings(repo_path: &Path) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    findings.extend(claude_repo_findings(repo_path));
    findings.extend(codex_repo_findings(repo_path));
    findings.extend(pi_repo_findings(repo_path));
    findings
}

fn latest_log_runtime_findings(runtime: &RuntimeConfig) -> Vec<DoctorFinding> {
    let latest_path = runtime.log_dir.join("latest.log");
    let Ok(contents) = fs::read_to_string(&latest_path) else {
        return vec![DoctorFinding::new(
            "latest-log-missing",
            DoctorSeverity::Info,
            DoctorArea::Runtime,
            format!(
                "No latest Foreman log was found at {}.",
                latest_path.display()
            ),
        )];
    };

    let mut findings = Vec::new();
    for (provider, summary) in parse_latest_log_summaries(&contents) {
        if summary.fallback_to_compatibility > 0 && summary.applied == 0 {
            findings.push(
                DoctorFinding::new(
                    format!("{}-latest-log-fallback", provider.filter_label()),
                    DoctorSeverity::Warn,
                    DoctorArea::Runtime,
                    format!(
                        "Latest run saw {} {} pane(s) fall back to compatibility with no native signals applied.",
                        summary.fallback_to_compatibility,
                        provider.label()
                    ),
                )
                .with_provider(provider)
                .with_detail(format!("source={}", latest_path.display()))
                .with_next_step("Run foreman --doctor inside a live tmux environment"),
            );
        }
        if summary.warnings > 0 {
            findings.push(
                DoctorFinding::new(
                    format!("{}-latest-log-warnings", provider.filter_label()),
                    DoctorSeverity::Warn,
                    DoctorArea::Runtime,
                    format!(
                        "Latest run logged {} native warning(s) for {}.",
                        summary.warnings,
                        provider.label()
                    ),
                )
                .with_provider(provider)
                .with_detail(format!("source={}", latest_path.display())),
            );
        }
    }

    findings
}

fn live_runtime_context(runtime: &RuntimeConfig) -> Result<Option<LiveRuntimeSnapshot>, String> {
    if run_capture("tmux", &["-V"]).is_err() {
        return Ok(None);
    }

    let tmux = TmuxAdapter::new(SystemTmuxBackend::new(runtime.tmux_socket.clone()));
    let mut inventory = tmux
        .load_inventory(runtime.capture_lines)
        .map_err(|error| error.to_string())?;
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

    Ok(Some(LiveRuntimeSnapshot {
        inventory,
        claude_native,
        codex_native,
        pi_native,
    }))
}

fn provider_runtime_findings(
    runtime: &RuntimeConfig,
    inventory: &Inventory,
    provider: HarnessKind,
    warnings: &[String],
    repo_scope: Option<&Path>,
) -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    let affected_panes = provider_panes(inventory, provider, repo_scope);
    if affected_panes.is_empty() {
        return findings;
    }
    let applied = affected_panes
        .iter()
        .filter(|pane| {
            pane.agent
                .as_ref()
                .is_some_and(|agent| agent.integration_mode == IntegrationMode::Native)
        })
        .count();
    let fallback_to_compatibility = affected_panes
        .iter()
        .filter(|pane| {
            pane.agent
                .as_ref()
                .is_some_and(|agent| agent.integration_mode == IntegrationMode::Compatibility)
        })
        .count();

    let preference = provider_preference(runtime, provider);
    if matches!(
        preference,
        crate::config::IntegrationPreference::Compatibility
    ) {
        findings.push(
            DoctorFinding::new(
                format!("{}-forced-compatibility", provider.filter_label()),
                DoctorSeverity::Info,
                DoctorArea::Runtime,
                format!(
                    "{} is intentionally running in compatibility mode.",
                    provider.label()
                ),
            )
            .with_provider(provider),
        );
    } else if fallback_to_compatibility > 0 && applied == 0 {
        let mut finding = DoctorFinding::new(
            format!("{}-no-native-signals", provider.filter_label()),
            DoctorSeverity::Warn,
            DoctorArea::Runtime,
            format!(
                "No native signals were observed for {} visible {} pane(s).",
                fallback_to_compatibility,
                provider.label()
            ),
        )
        .with_provider(provider)
        .with_next_step(format!("Run {}", doctor_command(repo_scope)));
        if let Some(native_dir) = provider_native_dir(runtime, provider) {
            if !native_dir.exists() {
                finding.push_evidence(format!("native_dir_missing={}", native_dir.display()));
            } else if dir_file_count(native_dir) == 0 {
                finding.push_evidence(format!("native_dir_empty={}", native_dir.display()));
            } else {
                finding.push_evidence(format!("native_dir={}", native_dir.display()));
            }
        }
        findings.push(finding);
    } else if applied > 0 {
        findings.push(
            DoctorFinding::new(
                format!("{}-native-signals-live", provider.filter_label()),
                DoctorSeverity::Ok,
                DoctorArea::Runtime,
                format!(
                    "Native signals are active for {} visible {} pane(s).",
                    applied,
                    provider.label()
                ),
            )
            .with_provider(provider),
        );
    }

    if repo_scope.is_none() {
        for warning in warnings {
            findings.push(
                DoctorFinding::new(
                    format!("{}-native-warning", provider.filter_label()),
                    DoctorSeverity::Warn,
                    DoctorArea::Runtime,
                    format!("{} reported a native signal warning.", provider.label()),
                )
                .with_provider(provider)
                .with_detail(warning.clone()),
            );
        }
    }

    let mut repo_roots = BTreeMap::<PathBuf, usize>::new();
    for pane in &affected_panes {
        if let Some(repo_root) = pane_repo_root(pane) {
            *repo_roots.entry(repo_root).or_default() += 1;
        }
        if pane
            .agent
            .as_ref()
            .is_some_and(|agent| agent.integration_mode == IntegrationMode::Compatibility)
        {
            if let Some(signature) = hook_error_signature(&pane.preview) {
                findings.push(
                    DoctorFinding::new(
                        format!("{}-hook-command-error", provider.filter_label()),
                        DoctorSeverity::Error,
                        DoctorArea::Runtime,
                        format!(
                            "{} pane likely has a failing hook command.",
                            provider.label()
                        ),
                    )
                    .with_provider(provider)
                    .with_pane_id(pane.id.as_str())
                    .with_detail(signature.to_string())
                    .with_next_step(format!(
                        "Run {}",
                        setup_command(pane_repo_root(pane).as_deref().or(repo_scope))
                    )),
                );
            }
        }
    }

    for repo_root in repo_roots.keys() {
        findings.extend(runtime_repo_degradation_findings(provider, repo_root));
    }

    findings
}

fn runtime_repo_degradation_findings(
    provider: HarnessKind,
    repo_root: &Path,
) -> Vec<DoctorFinding> {
    let findings = match provider {
        HarnessKind::ClaudeCode => claude_repo_findings(repo_root),
        HarnessKind::CodexCli => codex_repo_findings(repo_root),
        HarnessKind::Pi => pi_repo_findings(repo_root),
        HarnessKind::GeminiCli | HarnessKind::OpenCode => Vec::new(),
    };

    findings
        .into_iter()
        .filter(|finding| finding.severity != DoctorSeverity::Ok)
        .map(|mut finding| {
            finding.area = DoctorArea::Runtime;
            finding
        })
        .collect()
}

fn claude_repo_findings(repo_path: &Path) -> Vec<DoctorFinding> {
    let searched = claude_candidate_paths(repo_path);
    let mut parse_errors = Vec::new();
    let mut wired_paths = Vec::new();
    let mut non_hook_paths = Vec::new();

    for path in &searched {
        if !path.exists() {
            continue;
        }
        match json_file_contains_hook(path, "foreman-claude-hook") {
            Ok(true) => {
                wired_paths.push(path.display().to_string());
            }
            Ok(false) => {
                non_hook_paths.push(format!(
                    "{} exists but does not reference foreman-claude-hook",
                    path.display()
                ));
            }
            Err(error) => {
                parse_errors.push(format!("{} could not be parsed: {}", path.display(), error));
            }
        }
    }

    if !parse_errors.is_empty() {
        let mut finding = DoctorFinding::new(
            "claude-hook-invalid-json",
            DoctorSeverity::Error,
            DoctorArea::Repo,
            "Claude hook config could not be parsed in repo-local or global settings.",
        )
        .with_provider(HarnessKind::ClaudeCode)
        .with_repo_path(repo_path)
        .with_next_step(format!(
            "Repair invalid JSON, then run {}",
            setup_command(Some(repo_path))
        ));
        for path in searched {
            finding.push_evidence(format!("searched={}", path.display()));
        }
        for path in wired_paths {
            finding.push_evidence(format!(
                "{} also references foreman-claude-hook but invalid Claude settings remain",
                path
            ));
        }
        for path in non_hook_paths {
            finding.push_evidence(path);
        }
        for parse_error in parse_errors {
            finding.push_evidence(parse_error);
        }
        return vec![finding];
    }

    if let Some(path) = wired_paths.first() {
        return vec![DoctorFinding::new(
            "claude-hook-wired",
            DoctorSeverity::Ok,
            DoctorArea::Repo,
            format!("Claude hook wiring is present in {}.", path),
        )
        .with_provider(HarnessKind::ClaudeCode)
        .with_repo_path(repo_path)];
    }

    let mut finding = DoctorFinding::new(
        "claude-hook-missing",
        DoctorSeverity::Warn,
        DoctorArea::Repo,
        "Claude hook wiring was not found in repo-local or global settings.",
    )
    .with_provider(HarnessKind::ClaudeCode)
    .with_repo_path(repo_path)
    .with_next_step(format!(
        "Run {} for the safe fixes, then add foreman-claude-hook to one of: {}",
        setup_command(Some(repo_path)),
        searched
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ));
    for path in searched {
        finding.push_evidence(format!("searched={}", path.display()));
    }
    for path in non_hook_paths {
        finding.push_evidence(path);
    }

    vec![finding]
}

fn codex_repo_findings(repo_path: &Path) -> Vec<DoctorFinding> {
    let searched = codex_candidate_paths(repo_path);
    let mut parse_errors = Vec::new();
    let mut wired_paths = Vec::new();
    let mut non_hook_paths = Vec::new();

    for path in &searched {
        if !path.exists() {
            continue;
        }
        match json_file_contains_hook(path, "foreman-codex-hook") {
            Ok(true) => {
                wired_paths.push(path.display().to_string());
            }
            Ok(false) => {
                non_hook_paths.push(path.display().to_string());
            }
            Err(error) => {
                parse_errors.push(format!("{} could not be parsed: {}", path.display(), error));
            }
        }
    }

    let mut finding = DoctorFinding::new(
        if parse_errors.is_empty() {
            "codex-hook-file-missing"
        } else {
            "codex-hook-invalid-json"
        },
        if parse_errors.is_empty() {
            DoctorSeverity::Warn
        } else {
            DoctorSeverity::Error
        },
        DoctorArea::Repo,
        if parse_errors.is_empty() {
            "Codex hook wiring was not found in repo-local or user-level config.".to_string()
        } else {
            "Codex hook config could not be parsed in repo-local or user-level config.".to_string()
        },
    )
    .with_provider(HarnessKind::CodexCli)
    .with_repo_path(repo_path)
    .with_next_step(format!("Run {}", setup_command(Some(repo_path))));
    for path in searched {
        finding.push_evidence(format!("searched={}", path.display()));
    }
    for path in &wired_paths {
        finding.push_evidence(format!(
            "{} also references foreman-codex-hook but invalid Codex config remains",
            path
        ));
    }
    for path in &non_hook_paths {
        finding.push_evidence(format!(
            "{} exists but does not reference foreman-codex-hook",
            path
        ));
    }
    for parse_error in &parse_errors {
        finding.push_evidence(parse_error);
    }

    if parse_errors.is_empty() {
        if let Some(path) = wired_paths.first() {
            return vec![DoctorFinding::new(
                "codex-hook-wired",
                DoctorSeverity::Ok,
                DoctorArea::Repo,
                format!("Codex hook wiring is present in {}.", path),
            )
            .with_provider(HarnessKind::CodexCli)
            .with_repo_path(repo_path)];
        }
    }

    vec![finding]
}

fn pi_repo_findings(repo_path: &Path) -> Vec<DoctorFinding> {
    let searched = pi_candidate_paths(repo_path);
    let mut read_errors = Vec::new();
    let mut wired_paths = Vec::new();
    let mut non_hook_paths = Vec::new();

    for path in &searched {
        if !path.exists() {
            continue;
        }
        match fs::read_to_string(path) {
            Ok(contents) => {
                if contents.contains("foreman-pi-hook") {
                    wired_paths.push(path.display().to_string());
                } else {
                    non_hook_paths.push(path.display().to_string());
                }
            }
            Err(error) => {
                read_errors.push(format!("{} could not be read: {}", path.display(), error));
            }
        }
    }

    if read_errors.is_empty() {
        if let Some(path) = wired_paths.first() {
            return vec![DoctorFinding::new(
                "pi-extension-wired",
                DoctorSeverity::Ok,
                DoctorArea::Repo,
                format!("Pi Foreman extension is present in {}.", path),
            )
            .with_provider(HarnessKind::Pi)
            .with_repo_path(repo_path)];
        }
    }

    let mut finding = DoctorFinding::new(
        if read_errors.is_empty() {
            "pi-extension-missing"
        } else {
            "pi-extension-unreadable"
        },
        if read_errors.is_empty() {
            DoctorSeverity::Warn
        } else {
            DoctorSeverity::Error
        },
        DoctorArea::Repo,
        if read_errors.is_empty() {
            "Pi Foreman extension was not found in repo-local or user-level config.".to_string()
        } else {
            "Pi Foreman extension could not be read in repo-local or user-level config.".to_string()
        },
    )
    .with_provider(HarnessKind::Pi)
    .with_repo_path(repo_path)
    .with_next_step(format!("Run {}", setup_command(Some(repo_path))));
    for path in searched {
        finding.push_evidence(format!("searched={}", path.display()));
    }
    for path in wired_paths {
        finding.push_evidence(format!(
            "{} also references foreman-pi-hook but unreadable Pi config remains",
            path
        ));
    }
    for path in non_hook_paths {
        finding.push_evidence(format!(
            "{} exists but does not reference foreman-pi-hook",
            path
        ));
    }
    for read_error in read_errors {
        finding.push_evidence(read_error);
    }

    vec![finding]
}

fn apply_safe_fixes(
    runtime: &RuntimeConfig,
    repo_path: Option<&Path>,
    fix_mode: DoctorFixMode,
    setup_scopes: SetupScopeSelection,
    setup_providers: SetupProviderSelection,
) -> Result<Vec<DoctorFixResult>, DoctorError> {
    let mut fixes = Vec::new();
    let resolved_scopes = setup_scopes.resolved(repo_path.is_some());
    let home = home_dir();
    let provider_targets = provider_fix_targets(repo_path, home.as_deref(), resolved_scopes);

    if fix_mode.writes() {
        for target in &provider_targets {
            if setup_providers.includes(target.provider) {
                let _ = apply_provider_fix(target, DoctorFixMode::DryRun)?;
            }
        }
    }

    if !runtime.config_file.exists() {
        let preview = crate::config::default_config_toml();
        if fix_mode.writes() {
            write_default_config(&runtime.config_file)?;
        }
        fixes.push(DoctorFixResult {
            provider: None,
            path: runtime.config_file.clone(),
            status: if fix_mode.writes() {
                DoctorFixStatus::Written
            } else {
                DoctorFixStatus::Planned
            },
            message: "initialize default config".to_string(),
            preview: Some(preview),
        });
    }

    for (provider, native_dir) in [
        (
            HarnessKind::ClaudeCode,
            runtime.claude_native_dir.as_deref(),
        ),
        (HarnessKind::CodexCli, runtime.codex_native_dir.as_deref()),
        (HarnessKind::Pi, runtime.pi_native_dir.as_deref()),
    ] {
        if let Some(native_dir) = native_dir {
            let should_prepare_native_dir = resolved_scopes.user
                || repo_path.is_some_and(|repo_path| native_dir.starts_with(repo_path));
            if should_prepare_native_dir && setup_providers.includes(provider) {
                fixes.push(apply_native_dir_fix(provider, native_dir, fix_mode)?);
            }
        }
    }

    for target in &provider_targets {
        if setup_providers.includes(target.provider) {
            fixes.push(apply_provider_fix(target, fix_mode)?);
        }
    }

    if resolved_scopes.project && repo_path.is_none() {
        fixes.push(DoctorFixResult {
            provider: None,
            path: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            status: DoctorFixStatus::Skipped,
            message: "project-level setup requested but no repo was detected".to_string(),
            preview: None,
        });
    }

    if resolved_scopes.user && home.is_none() {
        fixes.push(DoctorFixResult {
            provider: None,
            path: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            status: DoctorFixStatus::Skipped,
            message: "user-level setup requested but no home directory was resolved".to_string(),
            preview: None,
        });
    }

    Ok(fixes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderFixKind {
    Claude,
    Codex,
    Pi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderFixTarget {
    provider: HarnessKind,
    kind: ProviderFixKind,
    path: PathBuf,
}

fn provider_fix_targets(
    repo_path: Option<&Path>,
    home: Option<&Path>,
    resolved_scopes: SetupScopeSelection,
) -> Vec<ProviderFixTarget> {
    let mut targets = Vec::new();

    if resolved_scopes.project {
        if let Some(repo_path) = repo_path {
            targets.push(ProviderFixTarget {
                provider: HarnessKind::ClaudeCode,
                kind: ProviderFixKind::Claude,
                path: repo_path.join(".claude").join("settings.local.json"),
            });
            targets.push(ProviderFixTarget {
                provider: HarnessKind::CodexCli,
                kind: ProviderFixKind::Codex,
                path: repo_path.join(".codex").join("hooks.json"),
            });
            targets.push(ProviderFixTarget {
                provider: HarnessKind::Pi,
                kind: ProviderFixKind::Pi,
                path: repo_path.join(".pi").join("extensions").join("foreman.ts"),
            });
        }
    }

    if resolved_scopes.user {
        if let Some(home) = home {
            targets.push(ProviderFixTarget {
                provider: HarnessKind::ClaudeCode,
                kind: ProviderFixKind::Claude,
                path: home.join(".claude").join("settings.local.json"),
            });
            targets.push(ProviderFixTarget {
                provider: HarnessKind::CodexCli,
                kind: ProviderFixKind::Codex,
                path: home.join(".codex").join("hooks.json"),
            });
            targets.push(ProviderFixTarget {
                provider: HarnessKind::Pi,
                kind: ProviderFixKind::Pi,
                path: home.join(".pi").join("extensions").join("foreman.ts"),
            });
        }
    }

    targets
}

fn apply_provider_fix(
    target: &ProviderFixTarget,
    fix_mode: DoctorFixMode,
) -> Result<DoctorFixResult, DoctorError> {
    match target.kind {
        ProviderFixKind::Claude => apply_claude_fix(&target.path, fix_mode),
        ProviderFixKind::Codex => apply_codex_fix(&target.path, fix_mode),
        ProviderFixKind::Pi => apply_pi_fix(&target.path, fix_mode),
    }
}

fn apply_native_dir_fix(
    provider: HarnessKind,
    path: &Path,
    fix_mode: DoctorFixMode,
) -> Result<DoctorFixResult, DoctorError> {
    let exists = path.exists();
    if !exists && fix_mode.writes() {
        fs::create_dir_all(path)?;
    }

    Ok(DoctorFixResult {
        provider: Some(provider),
        path: path.to_path_buf(),
        status: if exists {
            DoctorFixStatus::Unchanged
        } else if fix_mode.writes() {
            DoctorFixStatus::Written
        } else {
            DoctorFixStatus::Planned
        },
        message: if exists {
            format!(
                "{} native signal directory already exists",
                provider.label()
            )
        } else {
            format!("create {} native signal directory", provider.label())
        },
        preview: Some(format!("mkdir -p {}", path.display())),
    })
}

fn apply_codex_fix(path: &Path, fix_mode: DoctorFixMode) -> Result<DoctorFixResult, DoctorError> {
    let existing = if path.exists() {
        Some(fs::read_to_string(path)?)
    } else {
        None
    };

    let (value, changed) = merge_codex_hooks(existing.as_deref())?;
    let preview = serde_json::to_string_pretty(&value)?;
    if changed && fix_mode.writes() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("{preview}\n"))?;
    }

    Ok(DoctorFixResult {
        provider: Some(HarnessKind::CodexCli),
        path: path.to_path_buf(),
        status: if changed {
            if fix_mode.writes() {
                DoctorFixStatus::Written
            } else {
                DoctorFixStatus::Planned
            }
        } else {
            DoctorFixStatus::Unchanged
        },
        message: if changed {
            "ensure foreman-codex-hook entries for UserPromptSubmit and Stop".to_string()
        } else {
            "foreman-codex-hook entries already present".to_string()
        },
        preview: Some(preview),
    })
}

fn apply_pi_fix(path: &Path, fix_mode: DoctorFixMode) -> Result<DoctorFixResult, DoctorError> {
    let preview = pi_extension_template();
    let status = if path.exists() {
        let contents = fs::read_to_string(path)?;
        if contents.contains("foreman-pi-hook") {
            DoctorFixStatus::Unchanged
        } else {
            DoctorFixStatus::Skipped
        }
    } else if fix_mode.writes() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("{preview}\n"))?;
        DoctorFixStatus::Written
    } else {
        DoctorFixStatus::Planned
    };

    Ok(DoctorFixResult {
        provider: Some(HarnessKind::Pi),
        path: path.to_path_buf(),
        status,
        message: match status {
            DoctorFixStatus::Written | DoctorFixStatus::Planned => {
                "install Foreman Pi extension scaffold".to_string()
            }
            DoctorFixStatus::Unchanged => "Foreman Pi extension already present".to_string(),
            DoctorFixStatus::Skipped => {
                "existing Pi extension does not look like Foreman-managed code".to_string()
            }
        },
        preview: Some(preview),
    })
}

fn apply_claude_fix(path: &Path, fix_mode: DoctorFixMode) -> Result<DoctorFixResult, DoctorError> {
    let existing = if path.exists() {
        Some(fs::read_to_string(path)?)
    } else {
        None
    };
    let (value, changed) = merge_claude_hooks(existing.as_deref())?;
    let preview = serde_json::to_string_pretty(&value)?;
    if changed && fix_mode.writes() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, format!("{preview}\n"))?;
    }

    Ok(DoctorFixResult {
        provider: Some(HarnessKind::ClaudeCode),
        path: path.to_path_buf(),
        status: if changed {
            if fix_mode.writes() {
                DoctorFixStatus::Written
            } else {
                DoctorFixStatus::Planned
            }
        } else {
            DoctorFixStatus::Unchanged
        },
        message: if changed {
            "ensure foreman-claude-hook entries for UserPromptSubmit, Stop, StopFailure, and Notification".to_string()
        } else {
            "foreman-claude-hook entries already present".to_string()
        },
        preview: Some(preview),
    })
}

fn merge_claude_hooks(existing: Option<&str>) -> Result<(Value, bool), DoctorError> {
    let mut root = if let Some(existing) = existing {
        serde_json::from_str::<Value>(existing)?
    } else {
        json!({})
    };

    let Some(root_object) = root.as_object_mut() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "claude settings root is not a JSON object",
        ))
        .into());
    };
    let hooks_value = root_object
        .entry("hooks".to_string())
        .or_insert_with(|| json!({}));
    let Some(hooks_object) = hooks_value.as_object_mut() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "claude hooks value is not a JSON object",
        ))
        .into());
    };

    let mut changed = false;
    for event in ["UserPromptSubmit", "Stop", "StopFailure"] {
        let event_value = hooks_object
            .entry(event.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let already_present = value_contains_string(event_value, "foreman-claude-hook");
        let Some(array) = event_value.as_array_mut() else {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("claude hook event {event} is not an array"),
            ))
            .into());
        };
        if !already_present {
            array.push(json!({
                "hooks": [
                    {
                        "type": "command",
                        "command": "foreman-claude-hook"
                    }
                ]
            }));
            changed = true;
        }
    }

    let notification_value = hooks_object
        .entry("Notification".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let notification_present = value_contains_string(notification_value, "foreman-claude-hook");
    let Some(notification_array) = notification_value.as_array_mut() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "claude hook event Notification is not an array",
        ))
        .into());
    };
    if !notification_present {
        notification_array.push(json!({
            "matcher": "permission_prompt|elicitation_dialog",
            "hooks": [
                {
                    "type": "command",
                    "command": "foreman-claude-hook"
                }
            ]
        }));
        changed = true;
    }

    Ok((root, changed))
}

fn merge_codex_hooks(existing: Option<&str>) -> Result<(Value, bool), DoctorError> {
    let mut root = if let Some(existing) = existing {
        serde_json::from_str::<Value>(existing)?
    } else {
        json!({})
    };

    let Some(root_object) = root.as_object_mut() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "codex hooks root is not a JSON object",
        ))
        .into());
    };
    let hooks_value = root_object
        .entry("hooks".to_string())
        .or_insert_with(|| json!({}));
    let Some(hooks_object) = hooks_value.as_object_mut() else {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "codex hooks value is not a JSON object",
        ))
        .into());
    };

    let mut changed = false;
    for event in ["UserPromptSubmit", "Stop"] {
        let event_value = hooks_object
            .entry(event.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let already_present = value_contains_string(event_value, "foreman-codex-hook");
        let Some(array) = event_value.as_array_mut() else {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("codex hook event {event} is not an array"),
            ))
            .into());
        };
        if !already_present {
            array.push(json!({
                "hooks": [
                    {
                        "type": "command",
                        "command": "foreman-codex-hook"
                    }
                ]
            }));
            changed = true;
        }
    }

    Ok((root, changed))
}

fn provider_panes<'a>(
    inventory: &'a Inventory,
    provider: HarnessKind,
    repo_scope: Option<&Path>,
) -> Vec<&'a Pane> {
    inventory
        .sessions
        .iter()
        .flat_map(|session| session.windows.iter())
        .flat_map(|window| window.panes.iter())
        .filter(|pane| {
            pane.agent
                .as_ref()
                .is_some_and(|agent| agent.harness == provider)
        })
        .filter(|pane| match repo_scope {
            Some(repo_scope) => pane_repo_root(pane).as_deref() == Some(repo_scope),
            None => true,
        })
        .collect()
}

fn provider_preference(
    runtime: &RuntimeConfig,
    provider: HarnessKind,
) -> crate::config::IntegrationPreference {
    match provider {
        HarnessKind::ClaudeCode => runtime.claude_integration_preference,
        HarnessKind::CodexCli => runtime.codex_integration_preference,
        HarnessKind::Pi => runtime.pi_integration_preference,
        HarnessKind::GeminiCli | HarnessKind::OpenCode => {
            crate::config::IntegrationPreference::Compatibility
        }
    }
}

fn provider_native_dir(runtime: &RuntimeConfig, provider: HarnessKind) -> Option<&Path> {
    match provider {
        HarnessKind::ClaudeCode => runtime.claude_native_dir.as_deref(),
        HarnessKind::CodexCli => runtime.codex_native_dir.as_deref(),
        HarnessKind::Pi => runtime.pi_native_dir.as_deref(),
        HarnessKind::GeminiCli | HarnessKind::OpenCode => None,
    }
}

fn command_check(
    binary: &str,
    args: &[&str],
    provider: Option<HarnessKind>,
    area: DoctorArea,
) -> DoctorFinding {
    match run_capture(binary, args) {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut finding = DoctorFinding::new(
                format!("{binary}-available"),
                DoctorSeverity::Ok,
                area,
                format!(
                    "{} is available{}.",
                    binary,
                    stdout
                        .lines()
                        .next()
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| format!(" ({line})"))
                        .unwrap_or_default()
                ),
            );
            if let Some(provider) = provider {
                finding = finding.with_provider(provider);
            }
            finding
        }
        Ok(output) => {
            let mut finding = DoctorFinding::new(
                format!("{binary}-failed"),
                DoctorSeverity::Warn,
                area,
                format!("{binary} is on PATH but did not run successfully."),
            );
            if let Some(provider) = provider {
                finding = finding.with_provider(provider);
            }
            finding.with_detail(stderr_or_stdout(&output))
        }
        Err(_) => {
            let mut finding = DoctorFinding::new(
                format!("{binary}-missing"),
                DoctorSeverity::Warn,
                area,
                format!("{binary} is not available on PATH."),
            );
            if let Some(provider) = provider {
                finding = finding.with_provider(provider);
            }
            finding
        }
    }
}

fn codex_version_findings() -> Vec<DoctorFinding> {
    let mut findings = Vec::new();
    let Ok(version_output) = run_capture("codex", &["--version"]) else {
        return findings;
    };
    if !version_output.status.success() {
        return findings;
    }
    let version_text = String::from_utf8_lossy(&version_output.stdout).to_string();
    if let Some(version) = parse_semver(&version_text) {
        let severity = if version < MIN_CODEX_HOOK_VERSION {
            DoctorSeverity::Error
        } else {
            DoctorSeverity::Ok
        };
        findings.push(
            DoctorFinding::new(
                "codex-version-floor",
                severity,
                DoctorArea::Machine,
                format!(
                    "Codex CLI version is {}.{}.{}.",
                    version.0, version.1, version.2
                ),
            )
            .with_provider(HarnessKind::CodexCli)
            .with_next_step(format!(
                "Use Codex CLI >= {}.{}.{} for UserPromptSubmit hooks.",
                MIN_CODEX_HOOK_VERSION.0, MIN_CODEX_HOOK_VERSION.1, MIN_CODEX_HOOK_VERSION.2
            )),
        );
    }

    match run_capture("codex", &["features", "list"]) {
        Ok(output) if output.status.success() => findings.push(
            DoctorFinding::new(
                "codex-hook-feature",
                if String::from_utf8_lossy(&output.stdout).contains("codex_hooks") {
                    DoctorSeverity::Ok
                } else {
                    DoctorSeverity::Error
                },
                DoctorArea::Machine,
                if String::from_utf8_lossy(&output.stdout).contains("codex_hooks") {
                    "Codex exposes the codex_hooks feature.".to_string()
                } else {
                    "Codex does not expose the codex_hooks feature.".to_string()
                },
            )
            .with_provider(HarnessKind::CodexCli),
        ),
        Ok(output) => findings.push(
            DoctorFinding::new(
                "codex-feature-check-failed",
                DoctorSeverity::Warn,
                DoctorArea::Machine,
                "Codex feature inventory could not be read.".to_string(),
            )
            .with_provider(HarnessKind::CodexCli)
            .with_detail(stderr_or_stdout(&output)),
        ),
        Err(_) => {}
    }

    findings
}

fn parse_latest_log_summaries(contents: &str) -> Vec<(HarnessKind, LoggedNativeSummary)> {
    let mut by_provider = BTreeMap::new();
    for line in contents.lines() {
        for (prefix, provider) in [
            ("claude_native_summary", HarnessKind::ClaudeCode),
            ("codex_native_summary", HarnessKind::CodexCli),
            ("pi_native_summary", HarnessKind::Pi),
        ] {
            if line.contains(prefix) {
                by_provider.insert(provider, LoggedNativeSummary::from_line(line));
            }
        }
    }

    by_provider.into_iter().collect()
}

fn resolve_requested_repo_root(repo_path: Option<&Path>) -> Option<PathBuf> {
    let start = repo_path
        .map(Path::to_path_buf)
        .or_else(|| env::current_dir().ok())?;
    let repo_root = resolve_repo_root(&start);
    looks_like_repo_root(&repo_root).then_some(repo_root)
}

fn doctor_command(repo_path: Option<&Path>) -> String {
    scoped_command("foreman --doctor", repo_path)
}

fn setup_command(repo_path: Option<&Path>) -> String {
    scoped_command("foreman --setup", repo_path)
}

fn scoped_command(base: &str, repo_path: Option<&Path>) -> String {
    match repo_path {
        Some(path) if current_repo_root().as_deref() != Some(path) => {
            format!("{base} --repo {}", path.display())
        }
        _ => base.to_string(),
    }
}

fn current_repo_root() -> Option<PathBuf> {
    resolve_requested_repo_root(None)
}

fn resolve_repo_root(start: &Path) -> PathBuf {
    let mut cursor = start.to_path_buf();
    if cursor.is_file() {
        cursor.pop();
    }
    for candidate in cursor.ancestors() {
        if candidate.join(".git").exists() {
            return candidate.to_path_buf();
        }
    }
    cursor
}

fn looks_like_repo_root(path: &Path) -> bool {
    path.join(".git").exists()
}

fn pane_repo_root(pane: &Pane) -> Option<PathBuf> {
    pane.working_dir
        .as_deref()
        .map(resolve_repo_root)
        .filter(|path| looks_like_repo_root(path))
}

fn claude_candidate_paths(repo_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_unique_path(&mut paths, repo_path.join(".claude").join("settings.json"));
    push_unique_path(
        &mut paths,
        repo_path.join(".claude").join("settings.local.json"),
    );
    if let Some(home) = home_dir() {
        push_unique_path(&mut paths, home.join(".claude").join("settings.json"));
        push_unique_path(&mut paths, home.join(".claude").join("settings.local.json"));
    }
    paths
}

fn codex_candidate_paths(repo_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_unique_path(&mut paths, repo_path.join(".codex").join("hooks.json"));
    if let Some(home) = home_dir() {
        push_unique_path(&mut paths, home.join(".codex").join("hooks.json"));
    }
    paths
}

fn pi_candidate_paths(repo_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    push_unique_path(
        &mut paths,
        repo_path.join(".pi").join("extensions").join("foreman.ts"),
    );
    if let Some(home) = home_dir() {
        push_unique_path(
            &mut paths,
            home.join(".pi").join("extensions").join("foreman.ts"),
        );
    }
    paths
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn json_file_contains_hook(path: &Path, hook_binary: &str) -> Result<bool, String> {
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_str(&contents).map_err(|error| error.to_string())?;
    Ok(value_contains_string(&value, hook_binary))
}

fn value_contains_string(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text.contains(needle),
        Value::Array(items) => items.iter().any(|item| value_contains_string(item, needle)),
        Value::Object(map) => map
            .values()
            .any(|value| value_contains_string(value, needle)),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn hook_error_signature(preview: &str) -> Option<&'static str> {
    let lower = preview.to_ascii_lowercase();
    HOOK_ERROR_SIGNATURES
        .iter()
        .copied()
        .find(|needle| lower.contains(needle))
}

fn dir_file_count(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    entry
                        .file_type()
                        .map(|file_type| file_type.is_file())
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

fn run_capture(binary: &str, args: &[&str]) -> Result<std::process::Output, std::io::Error> {
    Command::new(binary).args(args).output()
}

fn stderr_or_stdout(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn parse_semver(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find(|part| part.matches('.').count() >= 2)?
        .split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn context_preference_label(preference: crate::config::IntegrationPreference) -> &'static str {
    preference.label()
}

fn sort_findings(findings: &mut [DoctorFinding]) {
    findings.sort_by(|left, right| {
        left.area
            .cmp(&right.area)
            .then_with(|| right.severity.cmp(&left.severity))
            .then_with(|| left.provider.cmp(&right.provider))
            .then_with(|| left.summary.cmp(&right.summary))
    });
}

fn pi_extension_template() -> String {
    r#"import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { spawnSync } from "node:child_process";

function runHook(event: string) {
  const args = ["--event", event];
  const paneId = process.env.TMUX_PANE;
  if (paneId) {
    args.push("--pane-id", paneId);
  }
  const result = spawnSync("foreman-pi-hook", args, { stdio: "inherit" });
  if ((result.status ?? 1) !== 0) {
    throw new Error(`foreman-pi-hook failed for ${event}`);
  }
}

export default function (pi: ExtensionAPI) {
  pi.on("agent_start", async () => {
    runHook("agent-start");
  });
  pi.on("agent_end", async () => {
    runHook("agent-end");
  });
  pi.on("session_shutdown", async () => {
    runHook("session-shutdown");
  });
}"#
    .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LoggedNativeSummary {
    applied: usize,
    fallback_to_compatibility: usize,
    warnings: usize,
}

impl LoggedNativeSummary {
    fn from_line(line: &str) -> Self {
        let mut summary = Self {
            applied: 0,
            fallback_to_compatibility: 0,
            warnings: 0,
        };
        for token in line.split_whitespace() {
            if let Some(value) = token.strip_prefix("applied=") {
                summary.applied = value.parse().unwrap_or_default();
            } else if let Some(value) = token.strip_prefix("fallback_to_compatibility=") {
                summary.fallback_to_compatibility = value.parse().unwrap_or_default();
            } else if let Some(value) = token.strip_prefix("warnings=") {
                summary.warnings = value.parse().unwrap_or_default();
            }
        }
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_claude_fix, apply_codex_fix, apply_pi_fix, apply_safe_fixes, claude_repo_findings,
        codex_repo_findings, merge_claude_hooks, merge_codex_hooks, parse_latest_log_summaries,
        pi_repo_findings, resolve_requested_repo_root, runtime_findings, runtime_findings_scoped,
        DoctorFixMode, DoctorFixStatus, LoggedNativeSummary, RuntimeDoctorContext,
        SetupProviderSelection, SetupScopeSelection,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, PaneBuilder, SessionBuilder,
        WindowBuilder,
    };
    use crate::config::{AppConfig, RuntimeConfig};
    use clap::Parser;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static HOME_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn sample_runtime(config_file: &std::path::Path, log_dir: &std::path::Path) -> RuntimeConfig {
        RuntimeConfig::from_sources(
            crate::config::AppPaths {
                config_file: config_file.to_path_buf(),
                log_dir: log_dir.to_path_buf(),
                startup_cache_dir: crate::config::default_startup_cache_dir(log_dir),
            },
            AppConfig::default(),
            &crate::cli::Cli::parse_from(["foreman"]),
        )
    }

    fn with_temp_home<T>(f: impl FnOnce(&std::path::Path) -> T) -> T {
        let _guard = HOME_ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let original_home = std::env::var_os("HOME");
        let temp_home = tempdir().expect("temp home should exist");
        std::env::set_var("HOME", temp_home.path());
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(temp_home.path())));
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        match result {
            Ok(value) => value,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    }

    #[test]
    fn codex_repo_findings_warn_when_hook_file_is_missing() {
        with_temp_home(|_| {
            let temp_dir = tempdir().expect("temp dir should exist");
            let findings = codex_repo_findings(temp_dir.path());

            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].id, "codex-hook-file-missing");
        });
    }

    #[test]
    fn merge_codex_hooks_is_idempotent() {
        let original = r#"{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-codex-hook"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-codex-hook"
          }
        ]
      }
    ]
  }
}"#;
        let (_, changed) = merge_codex_hooks(Some(original)).expect("merge should work");
        assert!(!changed);
    }

    #[test]
    fn codex_fix_creates_hooks_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join(".codex/hooks.json");
        let result = apply_codex_fix(&path, DoctorFixMode::Apply).expect("fix should work");

        assert_eq!(result.status, DoctorFixStatus::Written);
        let contents = std::fs::read_to_string(path).expect("hooks file should exist");
        assert!(contents.contains("foreman-codex-hook"));
    }

    #[test]
    fn pi_fix_creates_extension_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join(".pi/extensions/foreman.ts");
        let result = apply_pi_fix(&path, DoctorFixMode::Apply).expect("fix should work");

        assert_eq!(result.status, DoctorFixStatus::Written);
        let contents = std::fs::read_to_string(path).expect("extension should exist");
        assert!(contents.contains("foreman-pi-hook"));
    }

    #[test]
    fn claude_fix_creates_settings_local_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join(".claude/settings.local.json");
        let result = apply_claude_fix(&path, DoctorFixMode::Apply).expect("fix should work");

        assert_eq!(result.status, DoctorFixStatus::Written);
        let contents = std::fs::read_to_string(path).expect("settings should exist");
        assert!(contents.contains("foreman-claude-hook"));
        assert!(contents.contains("Notification"));
    }

    #[test]
    fn merge_claude_hooks_is_idempotent() {
        let original = r#"{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "StopFailure": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "permission_prompt|elicitation_dialog",
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ]
  }
}"#;
        let (_, changed) = merge_claude_hooks(Some(original)).expect("merge should work");
        assert!(!changed);
    }

    #[test]
    fn runtime_findings_detect_hook_error_preview() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let runtime = sample_runtime(
            &temp_dir.path().join("config.toml"),
            &temp_dir.path().join("logs"),
        );
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::CodexCli)
                    .status(AgentStatus::Error)
                    .integration_mode(IntegrationMode::Compatibility)
                    .working_dir(temp_dir.path())
                    .preview("PreToolUse:Bash hook error"),
            ),
        )]);
        let claude_summary = crate::integrations::ClaudeNativeOverlaySummary::default();
        let codex_summary = crate::integrations::CodexNativeOverlaySummary {
            applied: 0,
            fallback_to_compatibility: 1,
            warnings: Vec::new(),
        };
        let pi_summary = crate::integrations::PiNativeOverlaySummary::default();
        let context = RuntimeDoctorContext {
            runtime: &runtime,
            config_exists: false,
            inventory: &inventory,
            claude_native: &claude_summary,
            codex_native: &codex_summary,
            pi_native: &pi_summary,
        };

        let findings = runtime_findings(&context);

        assert!(findings
            .iter()
            .any(|finding| finding.id == "codex-hook-command-error"));
        assert!(findings
            .iter()
            .any(|finding| finding.id == "codex-no-native-signals"));
    }

    #[test]
    fn parse_latest_log_summaries_uses_last_seen_values() {
        let summaries = parse_latest_log_summaries(
            "[INFO] claude_native_summary applied=0 fallback_to_compatibility=2 warnings=0\n\
             [INFO] claude_native_summary applied=1 fallback_to_compatibility=0 warnings=1\n",
        );

        assert_eq!(
            summaries,
            vec![(
                HarnessKind::ClaudeCode,
                LoggedNativeSummary {
                    applied: 1,
                    fallback_to_compatibility: 0,
                    warnings: 1
                }
            )]
        );
    }

    #[test]
    fn claude_repo_findings_search_repo_and_global_paths() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(temp_dir.path().join(".claude")).expect("dir should exist");
        std::fs::write(
            temp_dir.path().join(".claude/settings.json"),
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"foreman-claude-hook"}]}]}}"#,
        )
        .expect("settings should be written");

        let findings = claude_repo_findings(temp_dir.path());

        assert_eq!(findings[0].id, "claude-hook-wired");
    }

    #[test]
    fn claude_repo_findings_report_invalid_json_even_when_local_override_is_wired() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::create_dir_all(temp_dir.path().join(".claude")).expect("dir should exist");
        std::fs::write(
            temp_dir.path().join(".claude/settings.json"),
            "{invalid-json",
        )
        .expect("settings should be written");
        std::fs::write(
            temp_dir.path().join(".claude/settings.local.json"),
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"foreman-claude-hook"}]}]}}"#,
        )
        .expect("settings local should be written");

        let findings = claude_repo_findings(temp_dir.path());

        assert_eq!(findings[0].id, "claude-hook-invalid-json");
    }

    #[test]
    fn resolve_requested_repo_root_requires_git() {
        with_temp_home(|home| {
            std::fs::create_dir_all(home.join(".codex")).expect("codex dir should exist");
            std::fs::create_dir_all(home.join(".claude")).expect("claude dir should exist");

            let repo_root = resolve_requested_repo_root(Some(home));

            assert!(repo_root.is_none());
        });
    }

    #[test]
    fn runtime_findings_scope_to_selected_repo() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let repo_alpha = temp_dir.path().join("alpha");
        let repo_beta = temp_dir.path().join("beta");
        std::fs::create_dir_all(repo_alpha.join(".git")).expect("alpha git dir should exist");
        std::fs::create_dir_all(repo_beta.join(".git")).expect("beta git dir should exist");

        let runtime = sample_runtime(
            &temp_dir.path().join("config.toml"),
            &temp_dir.path().join("logs"),
        );
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:agents").pane(
                    PaneBuilder::agent("%1", HarnessKind::CodexCli)
                        .status(AgentStatus::Error)
                        .integration_mode(IntegrationMode::Compatibility)
                        .working_dir(&repo_alpha)
                        .preview("PreToolUse:Bash hook error"),
                ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:agents").pane(
                    PaneBuilder::agent("%2", HarnessKind::CodexCli)
                        .status(AgentStatus::Working)
                        .integration_mode(IntegrationMode::Native)
                        .working_dir(&repo_beta)
                        .preview("working"),
                ),
            ),
        ]);
        let context = RuntimeDoctorContext {
            runtime: &runtime,
            config_exists: true,
            inventory: &inventory,
            claude_native: &crate::integrations::ClaudeNativeOverlaySummary::default(),
            codex_native: &crate::integrations::CodexNativeOverlaySummary::default(),
            pi_native: &crate::integrations::PiNativeOverlaySummary::default(),
        };

        let findings = runtime_findings_scoped(
            &context,
            Some(&repo_alpha),
            SetupProviderSelection {
                codex: true,
                ..SetupProviderSelection::default()
            },
        );

        assert!(findings
            .iter()
            .any(|finding| finding.id == "codex-no-native-signals"));
        assert!(findings
            .iter()
            .any(|finding| finding.id == "codex-hook-command-error"));
        assert!(!findings
            .iter()
            .any(|finding| finding.id == "codex-native-signals-live"));
    }

    #[test]
    fn project_scope_does_not_create_home_native_dirs() {
        with_temp_home(|home| {
            let repo_dir = tempdir().expect("repo dir should exist");
            std::fs::create_dir_all(repo_dir.path().join(".git")).expect("git dir should exist");
            let runtime = sample_runtime(
                &home.join(".config/foreman/config.toml"),
                &home.join(".local/state/foreman/logs"),
            );

            let fixes = apply_safe_fixes(
                &runtime,
                Some(repo_dir.path()),
                DoctorFixMode::Apply,
                SetupScopeSelection {
                    project: true,
                    user: false,
                },
                SetupProviderSelection {
                    codex: true,
                    ..SetupProviderSelection::default()
                },
            )
            .expect("project-only setup should succeed");

            assert!(fixes.iter().all(|fix| !fix.path.ends_with("codex-native")));
            assert!(repo_dir.path().join(".codex/hooks.json").exists());
            assert!(!home.join(".local/state/foreman/codex-native").exists());
        });
    }

    #[test]
    fn apply_safe_fixes_preflights_selected_provider_errors_before_writing() {
        with_temp_home(|home| {
            let repo_dir = tempdir().expect("repo dir should exist");
            std::fs::create_dir_all(repo_dir.path().join(".git")).expect("git dir should exist");
            let codex_hooks = repo_dir.path().join(".codex/hooks.json");
            std::fs::create_dir_all(codex_hooks.parent().expect("parent should exist"))
                .expect("codex dir should exist");
            std::fs::write(&codex_hooks, "{invalid-json").expect("hooks file should be written");

            let runtime = sample_runtime(
                &home.join(".config/foreman/config.toml"),
                &home.join(".local/state/foreman/logs"),
            );

            let error = apply_safe_fixes(
                &runtime,
                Some(repo_dir.path()),
                DoctorFixMode::Apply,
                SetupScopeSelection {
                    project: true,
                    user: false,
                },
                SetupProviderSelection {
                    codex: true,
                    pi: true,
                    ..SetupProviderSelection::default()
                },
            )
            .expect_err("invalid codex config should abort setup");

            assert!(!error.to_string().is_empty());
            assert!(!runtime.config_file.exists());
            assert!(!repo_dir.path().join(".pi/extensions/foreman.ts").exists());
            assert_eq!(
                std::fs::read_to_string(&codex_hooks).expect("codex hooks should remain untouched"),
                "{invalid-json"
            );
        });
    }

    #[test]
    fn pi_repo_findings_warn_when_extension_missing() {
        with_temp_home(|_| {
            let temp_dir = tempdir().expect("temp dir should exist");
            let findings = pi_repo_findings(temp_dir.path());

            assert_eq!(findings[0].id, "pi-extension-missing");
        });
    }
}
