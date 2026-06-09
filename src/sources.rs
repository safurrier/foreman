use crate::services::control_api::{ActionResponse, AgentsResponse};
use crate::source_snapshots::{SnapshotFreshness, SourceSnapshotStore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const LOCAL_SOURCE_ID: &str = "local";
pub const LOCAL_SOURCE_LABEL: &str = "Local";

fn default_source_scope() -> SourceScope {
    SourceScope::Current
}

fn default_query_timeout_ms() -> u64 {
    5_000
}

fn default_current_source_first() -> bool {
    true
}

fn default_group_by() -> SourceDisplayGroupBy {
    SourceDisplayGroupBy::Session
}

fn default_dedupe_sessions() -> bool {
    true
}

fn default_local_first() -> bool {
    true
}

fn default_foreman_binary() -> String {
    "foreman".to_string()
}

fn default_ssh_binary() -> String {
    "ssh".to_string()
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SourceScope {
    All,
    #[default]
    Current,
    Local,
}

impl SourceScope {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "all" => Ok(Self::All),
            "current" => Ok(Self::Current),
            "local" => Ok(Self::Local),
            _ => Err(format!("unknown source scope '{value}'")),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Current => "current",
            Self::Local => "local",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourcesConfig {
    pub default_scope: SourceScope,
    pub current_source_first: bool,
    pub query_timeout_ms: u64,
    #[serde(default)]
    pub display: SourcesDisplayConfig,
    #[serde(flatten)]
    pub entries: BTreeMap<String, SourceConfig>,
}

impl Default for SourcesConfig {
    fn default() -> Self {
        Self {
            default_scope: default_source_scope(),
            current_source_first: default_current_source_first(),
            query_timeout_ms: default_query_timeout_ms(),
            display: SourcesDisplayConfig::default(),
            entries: BTreeMap::new(),
        }
    }
}

impl SourcesConfig {
    pub fn enabled_sources(&self) -> Vec<(SourceId, SourceConfig)> {
        let mut sources = Vec::new();
        let local_config = self
            .entries
            .get(LOCAL_SOURCE_ID)
            .cloned()
            .unwrap_or_else(SourceConfig::local);
        if local_config.enabled() {
            sources.push((SourceId::local(), local_config));
        }
        for (id, config) in &self.entries {
            if id == LOCAL_SOURCE_ID || !config.enabled() {
                continue;
            }
            sources.push((SourceId::new(id.clone()), config.clone()));
        }
        sources
    }

    pub fn get(&self, id: &SourceId) -> Option<SourceConfig> {
        if id.as_str() == LOCAL_SOURCE_ID {
            return Some(
                self.entries
                    .get(LOCAL_SOURCE_ID)
                    .cloned()
                    .unwrap_or_else(SourceConfig::local),
            );
        }
        self.entries.get(id.as_str()).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceDisplayGroupBy {
    Session,
}

impl Default for SourceDisplayGroupBy {
    fn default() -> Self {
        default_group_by()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SourcesDisplayConfig {
    pub group_by: SourceDisplayGroupBy,
    pub dedupe_sessions: bool,
    pub local_first: bool,
}

impl Default for SourcesDisplayConfig {
    fn default() -> Self {
        Self {
            group_by: default_group_by(),
            dedupe_sessions: default_dedupe_sessions(),
            local_first: default_local_first(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceDisplayConfig {
    pub show_label: Option<bool>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SourceJumpConfig {
    #[serde(alias = "activate_command")]
    pub activation_command: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum SourceConfig {
    Local {
        #[serde(default = "default_local_label")]
        label: String,
        #[serde(default = "default_enabled")]
        enabled: bool,
        #[serde(default)]
        tmux_server_name: Option<String>,
        #[serde(default)]
        display: SourceDisplayConfig,
    },
    Ssh {
        label: String,
        host: String,
        #[serde(default = "default_foreman_binary")]
        foreman: String,
        #[serde(default)]
        tmux_server_name: Option<String>,
        #[serde(default)]
        tmux_socket: Option<PathBuf>,
        #[serde(default = "default_ssh_binary")]
        ssh: String,
        #[serde(default = "default_enabled")]
        enabled: bool,
        #[serde(default)]
        query_timeout_ms: Option<u64>,
        #[serde(default)]
        extra_ssh_args: Vec<String>,
        #[serde(default)]
        display: SourceDisplayConfig,
        #[serde(default)]
        jump: SourceJumpConfig,
    },
    Snapshot {
        label: String,
        path: PathBuf,
        #[serde(default = "default_enabled")]
        enabled: bool,
        #[serde(default)]
        display: SourceDisplayConfig,
    },
}

fn default_local_label() -> String {
    LOCAL_SOURCE_LABEL.to_string()
}

impl SourceConfig {
    pub fn local() -> Self {
        Self::Local {
            label: default_local_label(),
            enabled: true,
            tmux_server_name: None,
            display: SourceDisplayConfig::default(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Local { label, .. } | Self::Ssh { label, .. } | Self::Snapshot { label, .. } => {
                label
            }
        }
    }

    pub fn display_label(&self) -> String {
        match self.display().label.as_deref() {
            Some(label) if !label.trim().is_empty() => label.to_string(),
            _ => self.label().to_string(),
        }
    }

    pub fn show_label(&self, id: &SourceId) -> bool {
        self.display()
            .show_label
            .unwrap_or_else(|| id.as_str() != LOCAL_SOURCE_ID)
    }

    pub fn display(&self) -> &SourceDisplayConfig {
        match self {
            Self::Local { display, .. }
            | Self::Ssh { display, .. }
            | Self::Snapshot { display, .. } => display,
        }
    }

    pub fn jump(&self) -> Option<&SourceJumpConfig> {
        match self {
            Self::Local { .. } | Self::Snapshot { .. } => None,
            Self::Ssh { jump, .. } => Some(jump),
        }
    }

    pub fn kind(&self) -> SourceKind {
        match self {
            Self::Local { .. } => SourceKind::Local,
            Self::Ssh { .. } => SourceKind::Ssh,
            Self::Snapshot { .. } => SourceKind::Snapshot,
        }
    }

    pub fn enabled(&self) -> bool {
        match self {
            Self::Local { enabled, .. }
            | Self::Ssh { enabled, .. }
            | Self::Snapshot { enabled, .. } => *enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Local,
    Ssh,
    Snapshot,
}

impl SourceKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Ssh => "ssh",
            Self::Snapshot => "snapshot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn local() -> Self {
        Self::new(LOCAL_SOURCE_ID)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate(value: &str) -> Result<(), String> {
        if value.is_empty() {
            return Err("source id must not be empty".to_string());
        }
        if value.starts_with('-') {
            return Err("source id must not start with '-'".to_string());
        }
        if !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        {
            return Err(
                "source id may contain only ASCII letters, numbers, dash, underscore, and dot"
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourcePaneId {
    pub source_id: SourceId,
    pub pane_id: String,
}

impl SourcePaneId {
    pub fn new(source_id: SourceId, pane_id: impl Into<String>) -> Self {
        Self {
            source_id,
            pane_id: pane_id.into(),
        }
    }

    pub fn stable_id(&self) -> String {
        format!("source:{}:pane:{}", self.source_id.as_str(), self.pane_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDescriptor {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub enabled: bool,
    pub display_label: String,
    pub show_label: bool,
}

impl SourceDescriptor {
    pub fn new(id: &SourceId, config: &SourceConfig) -> Self {
        Self {
            id: id.as_str().to_string(),
            label: config.label().to_string(),
            kind: config.kind().label().to_string(),
            enabled: config.enabled(),
            display_label: config.display_label(),
            show_label: config.show_label(id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDiagnostic {
    pub level: String,
    pub code: String,
    pub source_id: String,
    pub source_label: String,
    pub source_kind: String,
    pub message: String,
    pub retryable: bool,
    pub duration_ms: Option<u64>,
    pub last_success_unix_ms: Option<u128>,
}

impl SourceDiagnostic {
    pub fn warning(
        descriptor: &SourceDescriptor,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
        duration_ms: Option<u64>,
    ) -> Self {
        Self {
            level: "warning".to_string(),
            code: code.into(),
            source_id: descriptor.id.clone(),
            source_label: descriptor.label.clone(),
            source_kind: descriptor.kind.clone(),
            message: message.into(),
            retryable,
            duration_ms,
            last_success_unix_ms: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceSnapshot {
    pub descriptor: SourceDescriptor,
    pub response: Option<AgentsResponse>,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub duration_ms: u64,
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSummary {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub enabled: bool,
    pub ok: bool,
    pub stale: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct AggregateSnapshot {
    pub response: AgentsResponse,
    pub sources: Vec<SourceSummary>,
    pub source_diagnostics: Vec<SourceDiagnostic>,
    pub partial_failure_count: usize,
}

pub type SourceResult<T> = Result<T, SourceError>;

#[derive(Debug, Clone)]
pub struct SourceError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl SourceError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }
}

pub trait ForemanSource {
    fn descriptor(&self) -> SourceDescriptor;
    fn agents(&self) -> SourceResult<AgentsResponse>;
    fn focus(&self, pane_id: &str) -> SourceResult<ActionResponse>;
    fn send(&self, pane_id: &str, text: &str) -> SourceResult<ActionResponse>;
    fn extensions(&self, pane_id: &str) -> SourceResult<Value>;
}

type AgentsFn = Box<dyn Fn() -> SourceResult<AgentsResponse> + Send + Sync>;
type FocusFn = Box<dyn Fn(&str) -> SourceResult<ActionResponse> + Send + Sync>;
type SendFn = Box<dyn Fn(&str, &str) -> SourceResult<ActionResponse> + Send + Sync>;
type ExtensionsFn = Box<dyn Fn(&str) -> SourceResult<Value> + Send + Sync>;

pub struct LocalSource {
    descriptor: SourceDescriptor,
    agents_fn: AgentsFn,
    focus_fn: FocusFn,
    send_fn: SendFn,
    extensions_fn: ExtensionsFn,
}

impl LocalSource {
    pub fn new(
        descriptor: SourceDescriptor,
        agents_fn: AgentsFn,
        focus_fn: FocusFn,
        send_fn: SendFn,
        extensions_fn: ExtensionsFn,
    ) -> Self {
        Self {
            descriptor,
            agents_fn,
            focus_fn,
            send_fn,
            extensions_fn,
        }
    }
}

impl ForemanSource for LocalSource {
    fn descriptor(&self) -> SourceDescriptor {
        self.descriptor.clone()
    }

    fn agents(&self) -> SourceResult<AgentsResponse> {
        (self.agents_fn)()
    }

    fn focus(&self, pane_id: &str) -> SourceResult<ActionResponse> {
        (self.focus_fn)(pane_id)
    }

    fn send(&self, pane_id: &str, text: &str) -> SourceResult<ActionResponse> {
        (self.send_fn)(pane_id, text)
    }

    fn extensions(&self, pane_id: &str) -> SourceResult<Value> {
        (self.extensions_fn)(pane_id)
    }
}

#[derive(Debug, Clone)]
pub struct SshSource {
    id: SourceId,
    config: SourceConfig,
    default_timeout_ms: u64,
}

impl SshSource {
    pub fn new(id: SourceId, config: SourceConfig, default_timeout_ms: u64) -> Self {
        Self {
            id,
            config,
            default_timeout_ms,
        }
    }

    fn timeout_ms(&self) -> u64 {
        match &self.config {
            SourceConfig::Ssh {
                query_timeout_ms, ..
            } => query_timeout_ms.unwrap_or(self.default_timeout_ms),
            _ => self.default_timeout_ms,
        }
    }

    fn run_remote(&self, probe_args: &[String], stdin: Option<&str>) -> SourceResult<String> {
        let SourceConfig::Ssh {
            host,
            foreman,
            ssh,
            extra_ssh_args,
            ..
        } = &self.config
        else {
            return Err(SourceError::new(
                "source.invalid-kind",
                "SSH source used with non-SSH config",
                false,
            ));
        };
        validate_ssh_source_value("host", host)?;
        validate_ssh_source_value("foreman", foreman)?;
        validate_ssh_source_value("ssh", ssh)?;
        for arg in extra_ssh_args {
            validate_ssh_source_value("extra_ssh_args", arg)?;
        }

        let remote_command = remote_command(foreman, probe_args);
        let mut command = Command::new(ssh);
        command
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg(format!(
                "ConnectTimeout={}",
                (self.timeout_ms() / 1000).max(1)
            ))
            .args(extra_ssh_args)
            .arg(host)
            .arg(remote_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if stdin.is_some() {
            command.stdin(Stdio::piped());
        }
        let mut child = command.spawn().map_err(|error| {
            SourceError::new(
                "source.ssh.spawn-failed",
                format!("failed to start ssh for {}: {error}", self.id),
                true,
            )
        })?;
        if let Some(input) = stdin {
            if let Some(mut child_stdin) = child.stdin.take() {
                child_stdin.write_all(input.as_bytes()).map_err(|error| {
                    SourceError::new(
                        "source.ssh.stdin-failed",
                        format!("failed to write stdin for {}: {error}", self.id),
                        true,
                    )
                })?;
            }
        }
        let deadline = Instant::now() + Duration::from_millis(self.timeout_ms().max(1));
        loop {
            match child.try_wait().map_err(|error| {
                SourceError::new(
                    "source.ssh.wait-failed",
                    format!("failed to poll ssh for {}: {error}", self.id),
                    true,
                )
            })? {
                Some(_) => break,
                None if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(SourceError::new(
                        "source.ssh.timeout",
                        format!(
                            "remote source {} timed out after {}ms",
                            self.id,
                            self.timeout_ms()
                        ),
                        true,
                    ));
                }
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
        let output = child.wait_with_output().map_err(|error| {
            SourceError::new(
                "source.ssh.wait-failed",
                format!("failed to collect ssh output for {}: {error}", self.id),
                true,
            )
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(SourceError::new(
                "source.ssh.command-failed",
                format!("remote source {} failed: {stderr}", self.id),
                true,
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn base_probe_args(&self, command: &str) -> Vec<String> {
        let mut args = Vec::new();
        if let SourceConfig::Ssh {
            tmux_server_name,
            tmux_socket,
            ..
        } = &self.config
        {
            if let Some(name) = tmux_server_name {
                args.push("--tmux-server-name".to_string());
                args.push(name.clone());
            }
            if let Some(socket) = tmux_socket {
                args.push("--tmux-socket".to_string());
                args.push(socket.display().to_string());
            }
        }
        args.extend([
            "source-probe".to_string(),
            "--local-only".to_string(),
            command.to_string(),
        ]);
        args
    }
}

impl ForemanSource for SshSource {
    fn descriptor(&self) -> SourceDescriptor {
        SourceDescriptor::new(&self.id, &self.config)
    }

    fn agents(&self) -> SourceResult<AgentsResponse> {
        let mut args = self.base_probe_args("agents");
        args.push("--json".to_string());
        let stdout = self.run_remote(&args, None)?;
        let response: AgentsResponse = serde_json::from_str(&stdout).map_err(|error| {
            SourceError::new(
                "source.remote-schema.invalid-json",
                format!(
                    "remote source {} returned invalid agents JSON: {error}",
                    self.id
                ),
                false,
            )
        })?;
        if response.schema_version != crate::services::control_api::CONTROL_API_SCHEMA_VERSION {
            return Err(SourceError::new(
                "source.remote-schema.unsupported",
                format!(
                    "remote source {} returned unsupported agents schema version {}",
                    self.id, response.schema_version
                ),
                false,
            ));
        }
        Ok(response)
    }

    fn focus(&self, pane_id: &str) -> SourceResult<ActionResponse> {
        let mut args = self.base_probe_args("focus");
        args.extend([
            "--pane".to_string(),
            pane_id.to_string(),
            "--json".to_string(),
        ]);
        let stdout = self.run_remote(&args, None)?;
        let response: ActionResponse = serde_json::from_str(&stdout).map_err(|error| {
            SourceError::new(
                "source.remote-schema.invalid-json",
                format!(
                    "remote source {} returned invalid focus JSON: {error}",
                    self.id
                ),
                false,
            )
        })?;
        validate_action_schema(&self.id, "focus", response)
    }

    fn send(&self, pane_id: &str, text: &str) -> SourceResult<ActionResponse> {
        let mut args = self.base_probe_args("send");
        args.extend([
            "--pane".to_string(),
            pane_id.to_string(),
            "--stdin".to_string(),
            "--json".to_string(),
        ]);
        let stdout = self.run_remote(&args, Some(text))?;
        let response: ActionResponse = serde_json::from_str(&stdout).map_err(|error| {
            SourceError::new(
                "source.remote-schema.invalid-json",
                format!(
                    "remote source {} returned invalid send JSON: {error}",
                    self.id
                ),
                false,
            )
        })?;
        validate_action_schema(&self.id, "send", response)
    }

    fn extensions(&self, pane_id: &str) -> SourceResult<Value> {
        let mut args = self.base_probe_args("extensions");
        args.extend([
            "--pane".to_string(),
            pane_id.to_string(),
            "--json".to_string(),
        ]);
        let stdout = self.run_remote(&args, None)?;
        serde_json::from_str(&stdout).map_err(|error| {
            SourceError::new(
                "source.remote-schema.invalid-json",
                format!(
                    "remote source {} returned invalid extension JSON: {error}",
                    self.id
                ),
                false,
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotSource {
    id: SourceId,
    descriptor: SourceDescriptor,
    path: PathBuf,
}

impl SnapshotSource {
    pub fn new(id: SourceId, config: SourceConfig) -> Self {
        let descriptor = SourceDescriptor::new(&id, &config);
        let path = match config {
            SourceConfig::Snapshot { path, .. } => path,
            _ => PathBuf::new(),
        };
        Self {
            id,
            descriptor,
            path,
        }
    }

    fn unsupported_action(&self, action: &str) -> SourceError {
        SourceError::new(
            "source.snapshot.action-unsupported",
            format!(
                "snapshot source {} does not support {action}; use its live source transport",
                self.id
            ),
            false,
        )
    }
}

impl ForemanSource for SnapshotSource {
    fn descriptor(&self) -> SourceDescriptor {
        self.descriptor.clone()
    }

    fn agents(&self) -> SourceResult<AgentsResponse> {
        let store = SourceSnapshotStore::new(
            self.path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(".")),
        );
        let now_ms = unix_ms_now();
        let loaded =
            store.load_snapshot_for_source(&self.id, &self.descriptor, &self.path, now_ms)?;
        let mut response = loaded.envelope.response;
        if let Some(diagnostic) = SourceSnapshotStore::stale_diagnostic(
            &self.descriptor,
            loaded.freshness,
            loaded.envelope.captured_at_unix_ms,
            now_ms,
        ) {
            response.source_diagnostics.push(diagnostic);
            response.partial_failure_count += 1;
        }
        if loaded.freshness == SnapshotFreshness::Warm {
            response.source_diagnostics.push(SourceDiagnostic::warning(
                &self.descriptor,
                "source.snapshot.warm",
                format!(
                    "{} snapshot is warm: age={}ms",
                    self.descriptor.label,
                    now_ms.saturating_sub(loaded.envelope.captured_at_unix_ms)
                ),
                true,
                None,
            ));
        }
        Ok(response)
    }

    fn focus(&self, _pane_id: &str) -> SourceResult<ActionResponse> {
        Err(self.unsupported_action("focus"))
    }

    fn send(&self, _pane_id: &str, _text: &str) -> SourceResult<ActionResponse> {
        Err(self.unsupported_action("send"))
    }

    fn extensions(&self, _pane_id: &str) -> SourceResult<Value> {
        Err(self.unsupported_action("extensions"))
    }
}

fn validate_action_schema(
    source_id: &SourceId,
    action: &str,
    response: ActionResponse,
) -> SourceResult<ActionResponse> {
    if response.schema_version != crate::services::control_api::CONTROL_API_SCHEMA_VERSION {
        return Err(SourceError::new(
            "source.remote-schema.unsupported",
            format!(
                "remote source {source_id} returned unsupported {action} schema version {}",
                response.schema_version
            ),
            false,
        ));
    }
    Ok(response)
}

fn validate_ssh_source_value(field: &str, value: &str) -> SourceResult<()> {
    if value.is_empty() {
        return Err(SourceError::new(
            "source.ssh.invalid-config",
            format!("SSH source {field} must not be empty"),
            false,
        ));
    }
    if value.starts_with('-') {
        return Err(SourceError::new(
            "source.ssh.invalid-config",
            format!("SSH source {field} must not start with '-'"),
            false,
        ));
    }
    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(SourceError::new(
            "source.ssh.invalid-config",
            format!("SSH source {field} must not contain control line breaks"),
            false,
        ));
    }
    Ok(())
}

fn remote_command(foreman: &str, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(shell_quote(foreman));
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    parts.join(" ")
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':' | '='))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub struct SourceAggregator {
    sources: Vec<Box<dyn ForemanSource + Send + Sync>>,
}

impl SourceAggregator {
    pub fn new(sources: Vec<Box<dyn ForemanSource + Send + Sync>>) -> Self {
        Self { sources }
    }

    pub fn aggregate(&self) -> AggregateSnapshot {
        let mut aggregate_response: Option<AgentsResponse> = None;
        let mut source_summaries = Vec::new();
        let mut source_diagnostics = Vec::new();
        let mut partial_failure_count = 0;

        let results = std::thread::scope(|scope| {
            self.sources
                .iter()
                .map(|source| {
                    scope.spawn(move || {
                        let descriptor = source.descriptor();
                        let started = Instant::now();
                        let result = source.agents();
                        (descriptor, started.elapsed().as_millis() as u64, result)
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join())
                .collect::<Vec<_>>()
        });

        for result in results {
            match result {
                Ok((descriptor, duration_ms, Ok(mut response))) => {
                    let stale = response.source_diagnostics.iter().any(|diagnostic| {
                        matches!(
                            diagnostic.code.as_str(),
                            "source.snapshot.stale" | "source.snapshot.warm"
                        )
                    });
                    response.wrap_source(&descriptor);
                    if let Some(aggregate) = &mut aggregate_response {
                        aggregate.merge(response);
                    } else {
                        aggregate_response = Some(response);
                    }
                    source_summaries.push(SourceSummary {
                        id: descriptor.id,
                        label: descriptor.label,
                        kind: descriptor.kind,
                        enabled: descriptor.enabled,
                        ok: true,
                        stale,
                        duration_ms,
                    });
                }
                Ok((descriptor, duration_ms, Err(error))) => {
                    partial_failure_count += 1;
                    source_diagnostics.push(SourceDiagnostic::warning(
                        &descriptor,
                        error.code,
                        error.message,
                        error.retryable,
                        Some(duration_ms),
                    ));
                    source_summaries.push(SourceSummary {
                        id: descriptor.id,
                        label: descriptor.label,
                        kind: descriptor.kind,
                        enabled: descriptor.enabled,
                        ok: false,
                        stale: false,
                        duration_ms,
                    });
                }
                Err(_) => {
                    partial_failure_count += 1;
                    source_diagnostics.push(SourceDiagnostic {
                        level: "warning".to_string(),
                        code: "source.thread.panicked".to_string(),
                        source_id: "unknown".to_string(),
                        source_label: "unknown".to_string(),
                        source_kind: "unknown".to_string(),
                        message: "source query worker panicked".to_string(),
                        retryable: true,
                        duration_ms: None,
                        last_success_unix_ms: None,
                    });
                }
            }
        }

        let mut response = aggregate_response.unwrap_or_else(AgentsResponse::empty);
        response
            .source_diagnostics
            .extend(source_diagnostics.clone());
        response.sources = source_summaries.clone();
        response.partial_failure_count += partial_failure_count;
        let merged_source_diagnostics = response.source_diagnostics.clone();
        let merged_partial_failure_count = response.partial_failure_count;
        AggregateSnapshot {
            response,
            sources: source_summaries,
            source_diagnostics: merged_source_diagnostics,
            partial_failure_count: merged_partial_failure_count,
        }
    }
}

pub fn unix_ms_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::control_api::{AgentEntry, ControlInventorySummary};
    use std::sync::{Arc, Mutex};

    static SSH_TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn response(pane_id: &str) -> AgentsResponse {
        AgentsResponse {
            schema_version: crate::services::control_api::CONTROL_API_SCHEMA_VERSION,
            generated_at_unix_ms: 1,
            inventory: ControlInventorySummary {
                total_sessions: 1,
                total_windows: 1,
                total_panes: 1,
                visible_sessions: 1,
                visible_windows: 1,
                visible_panes: 1,
            },
            entries: vec![AgentEntry::test_entry(pane_id)],
            diagnostics: Vec::new(),
            sources: Vec::new(),
            source_diagnostics: Vec::new(),
            partial_failure_count: 0,
        }
    }

    #[test]
    fn source_pane_id_formats_stable_composite_id() {
        let id = SourcePaneId::new(SourceId::new("coder"), "%42");
        assert_eq!(id.stable_id(), "source:coder:pane:%42");
    }

    #[test]
    fn aggregator_wraps_duplicate_pane_ids_with_distinct_source_identity() {
        let left = LocalSource::new(
            SourceDescriptor {
                id: "local".to_string(),
                label: "Local".to_string(),
                kind: "local".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| Ok(response("%42"))),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        let right = LocalSource::new(
            SourceDescriptor {
                id: "coder".to_string(),
                label: "Coder".to_string(),
                kind: "ssh".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| Ok(response("%42"))),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        let aggregate = SourceAggregator::new(vec![Box::new(left), Box::new(right)]).aggregate();
        let ids: Vec<_> = aggregate
            .response
            .entries
            .iter()
            .map(|entry| entry.source_pane_id.as_str())
            .collect();
        assert_eq!(ids, vec!["source:local:pane:%42", "source:coder:pane:%42"]);
    }

    #[test]
    fn aggregator_queries_sources_in_parallel() {
        let slow_left = LocalSource::new(
            SourceDescriptor {
                id: "left".to_string(),
                label: "Left".to_string(),
                kind: "local".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| {
                std::thread::sleep(Duration::from_millis(150));
                Ok(response("%1"))
            }),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        let slow_right = LocalSource::new(
            SourceDescriptor {
                id: "right".to_string(),
                label: "Right".to_string(),
                kind: "local".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| {
                std::thread::sleep(Duration::from_millis(150));
                Ok(response("%2"))
            }),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );

        let started = Instant::now();
        let aggregate =
            SourceAggregator::new(vec![Box::new(slow_left), Box::new(slow_right)]).aggregate();

        assert_eq!(aggregate.response.entries.len(), 2);
        assert!(
            started.elapsed() < Duration::from_millis(275),
            "sources should be queried concurrently"
        );
    }

    #[test]
    fn aggregator_keeps_successful_sources_when_one_fails() {
        let good = LocalSource::new(
            SourceDescriptor {
                id: "local".to_string(),
                label: "Local".to_string(),
                kind: "local".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| Ok(response("%1"))),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        let bad = LocalSource::new(
            SourceDescriptor {
                id: "coder".to_string(),
                label: "Coder".to_string(),
                kind: "ssh".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| Err(SourceError::new("source.ssh.timeout", "timeout", true))),
            Box::new(|_| unreachable!()),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        let aggregate = SourceAggregator::new(vec![Box::new(good), Box::new(bad)]).aggregate();
        assert_eq!(aggregate.response.entries.len(), 1);
        assert_eq!(aggregate.partial_failure_count, 1);
        assert_eq!(aggregate.source_diagnostics[0].code, "source.ssh.timeout");
    }

    #[test]
    fn remote_command_quotes_shell_arguments() {
        let command = remote_command(
            "/path with spaces/foreman",
            &["source-probe".into(), "weird'arg;$(nope)".into()],
        );
        assert_eq!(
            command,
            "'/path with spaces/foreman' source-probe 'weird'\\''arg;$(nope)'"
        );
    }

    #[test]
    fn ssh_source_uses_non_recursive_source_probe() {
        let _guard = SSH_TEST_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let log = dir.path().join("ssh.log");
        let ssh = dir.path().join("ssh");
        std::fs::write(
            &ssh,
            format!(
                r#"#!/bin/sh
printf '%s\n' "$@" > {}
cat <<'JSON'
{{"schemaVersion":2,"generatedAtUnixMs":1,"inventory":{{"totalSessions":0,"totalWindows":0,"totalPanes":0,"visibleSessions":0,"visibleWindows":0,"visiblePanes":0}},"entries":[],"diagnostics":[]}}
JSON
"#,
                shell_quote(&log.display().to_string())
            ),
        )
        .expect("write fake ssh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&ssh).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&ssh, perms).unwrap();
        }
        let source = SshSource::new(
            SourceId::new("coder"),
            SourceConfig::Ssh {
                label: "Coder".to_string(),
                host: "coder.example".to_string(),
                foreman: "/usr/bin/foreman".to_string(),
                tmux_server_name: Some("user".to_string()),
                tmux_socket: None,
                ssh: ssh.display().to_string(),
                enabled: true,
                query_timeout_ms: Some(1_000),
                extra_ssh_args: Vec::new(),
                display: SourceDisplayConfig::default(),
                jump: SourceJumpConfig::default(),
            },
            1_000,
        );
        source.agents().expect("agents");
        let log = std::fs::read_to_string(log).expect("log");
        assert!(log.contains("coder.example"));
        assert!(log.contains("source-probe"));
        assert!(log.contains("--local-only"));
        assert!(log.contains("--tmux-server-name"));
    }

    #[test]
    fn ssh_source_reports_unsupported_remote_schema() {
        let _guard = SSH_TEST_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let ssh = dir.path().join("ssh");
        std::fs::write(
            &ssh,
            "#!/bin/sh\nprintf '{\"schemaVersion\":999,\"generatedAtUnixMs\":1,\"inventory\":{\"totalSessions\":0,\"totalWindows\":0,\"totalPanes\":0,\"visibleSessions\":0,\"visibleWindows\":0,\"visiblePanes\":0},\"entries\":[],\"diagnostics\":[]}'\n",
        )
        .expect("write fake ssh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&ssh).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&ssh, perms).unwrap();
        }
        let source = SshSource::new(
            SourceId::new("coder"),
            SourceConfig::Ssh {
                label: "Coder".to_string(),
                host: "coder.example".to_string(),
                foreman: "foreman".to_string(),
                tmux_server_name: None,
                tmux_socket: None,
                ssh: ssh.display().to_string(),
                enabled: true,
                query_timeout_ms: Some(1_000),
                extra_ssh_args: Vec::new(),
                display: SourceDisplayConfig::default(),
                jump: SourceJumpConfig::default(),
            },
            1_000,
        );

        let error = source.agents().expect_err("unsupported schema should fail");

        assert_eq!(error.code, "source.remote-schema.unsupported");
        assert!(error.message.contains("999"));
    }

    #[test]
    fn ssh_source_rejects_leading_dash_host() {
        let source = SshSource::new(
            SourceId::new("bad"),
            SourceConfig::Ssh {
                label: "Bad".to_string(),
                host: "-oProxyCommand=evil".to_string(),
                foreman: "foreman".to_string(),
                tmux_server_name: None,
                tmux_socket: None,
                ssh: "ssh".to_string(),
                enabled: true,
                query_timeout_ms: Some(100),
                extra_ssh_args: Vec::new(),
                display: SourceDisplayConfig::default(),
                jump: SourceJumpConfig::default(),
            },
            100,
        );

        let error = source.agents().expect_err("leading dash host rejected");

        assert_eq!(error.code, "source.ssh.invalid-config");
        assert!(error.message.contains("must not start"));
    }

    #[test]
    fn ssh_source_times_out_hung_remote_command() {
        let _guard = SSH_TEST_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let ssh = dir.path().join("ssh");
        std::fs::write(&ssh, "#!/bin/sh\nsleep 2\n").expect("write fake ssh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&ssh).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&ssh, perms).unwrap();
        }
        let source = SshSource::new(
            SourceId::new("coder"),
            SourceConfig::Ssh {
                label: "Coder".to_string(),
                host: "coder.example".to_string(),
                foreman: "/usr/bin/foreman".to_string(),
                tmux_server_name: Some("user".to_string()),
                tmux_socket: None,
                ssh: ssh.display().to_string(),
                enabled: true,
                query_timeout_ms: Some(50),
                extra_ssh_args: Vec::new(),
                display: SourceDisplayConfig::default(),
                jump: SourceJumpConfig::default(),
            },
            50,
        );
        let error = source.agents().expect_err("hung command should time out");
        assert_eq!(error.code, "source.ssh.timeout");
    }

    #[test]
    fn source_id_validation_rejects_shell_metacharacters() {
        assert!(SourceId::validate("coder-dev").is_ok());
        assert!(SourceId::validate("bad;rm").is_err());
        assert!(SourceId::validate("-bad").is_err());
    }

    #[test]
    fn local_source_actions_route_through_callbacks() {
        let focused = Arc::new(Mutex::new(Vec::new()));
        let focus_log = Arc::clone(&focused);
        let source = LocalSource::new(
            SourceDescriptor {
                id: "local".to_string(),
                label: "Local".to_string(),
                kind: "local".to_string(),
                enabled: true,
                display_label: "Local".to_string(),
                show_label: false,
            },
            Box::new(|| Ok(response("%1"))),
            Box::new(move |pane| {
                focus_log.lock().unwrap().push(pane.to_string());
                Ok(ActionResponse::new_focus("%1"))
            }),
            Box::new(|_, _| unreachable!()),
            Box::new(|_| unreachable!()),
        );
        source.focus("%1").unwrap();
        assert_eq!(&*focused.lock().unwrap(), &["%1".to_string()]);
    }
}
