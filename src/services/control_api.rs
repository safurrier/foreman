use crate::app::{AgentStatus, AppState, HarnessKind, IntegrationMode, Pane, PreviewProvenance};
use crate::services::extensions::ControlExtensionCard;
use crate::services::pull_requests::{PullRequestData, PullRequestStatus};
use crate::sources::{SourceDescriptor, SourceDiagnostic, SourceId, SourcePaneId, SourceSummary};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CONTROL_API_SCHEMA_VERSION: u16 = 2;
const MAX_PREVIEW_CHARS: usize = 1200;

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn default_source_id() -> String {
    crate::sources::LOCAL_SOURCE_ID.to_string()
}

fn default_source_label() -> String {
    crate::sources::LOCAL_SOURCE_LABEL.to_string()
}

fn default_source_kind() -> String {
    "local".to_string()
}

fn default_source_display_label() -> String {
    default_source_label()
}

fn default_source_show_label() -> bool {
    false
}

fn default_source_pane_id() -> String {
    String::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsResponse {
    pub schema_version: u16,
    pub generated_at_unix_ms: u128,
    pub inventory: ControlInventorySummary,
    pub entries: Vec<AgentEntry>,
    pub diagnostics: Vec<ControlDiagnostic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<SourceSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_diagnostics: Vec<SourceDiagnostic>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub partial_failure_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlInventorySummary {
    pub total_sessions: usize,
    pub total_windows: usize,
    pub total_panes: usize,
    pub visible_sessions: usize,
    pub visible_windows: usize,
    pub visible_panes: usize,
}

impl AgentsResponse {
    pub fn empty() -> Self {
        Self {
            schema_version: CONTROL_API_SCHEMA_VERSION,
            generated_at_unix_ms: generated_at_unix_ms(),
            inventory: ControlInventorySummary {
                total_sessions: 0,
                total_windows: 0,
                total_panes: 0,
                visible_sessions: 0,
                visible_windows: 0,
                visible_panes: 0,
            },
            entries: Vec::new(),
            diagnostics: Vec::new(),
            sources: Vec::new(),
            source_diagnostics: Vec::new(),
            partial_failure_count: 0,
        }
    }

    pub fn wrap_source(&mut self, descriptor: &SourceDescriptor) {
        for entry in &mut self.entries {
            entry.source_id = descriptor.id.clone();
            entry.source_label = descriptor.label.clone();
            entry.source_display_label = descriptor.display_label.clone();
            entry.source_show_label = descriptor.show_label;
            entry.source_kind = descriptor.kind.clone();
            let source_pane_id =
                SourcePaneId::new(SourceId::new(descriptor.id.clone()), entry.pane_id.clone());
            entry.source_pane_id = source_pane_id.stable_id();
            entry.id = entry.source_pane_id.clone();
        }
    }

    pub fn merge(&mut self, mut other: AgentsResponse) {
        self.inventory.total_sessions += other.inventory.total_sessions;
        self.inventory.total_windows += other.inventory.total_windows;
        self.inventory.total_panes += other.inventory.total_panes;
        self.inventory.visible_sessions += other.inventory.visible_sessions;
        self.inventory.visible_windows += other.inventory.visible_windows;
        self.inventory.visible_panes += other.inventory.visible_panes;
        self.entries.append(&mut other.entries);
        self.diagnostics.append(&mut other.diagnostics);
        self.sources.append(&mut other.sources);
        self.source_diagnostics
            .append(&mut other.source_diagnostics);
        self.partial_failure_count += other.partial_failure_count;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEntry {
    pub id: String,
    #[serde(default = "default_source_pane_id")]
    pub source_pane_id: String,
    #[serde(default = "default_source_id")]
    pub source_id: String,
    #[serde(default = "default_source_label")]
    pub source_label: String,
    #[serde(default = "default_source_display_label")]
    pub source_display_label: String,
    #[serde(default = "default_source_show_label")]
    pub source_show_label: bool,
    #[serde(default = "default_source_kind")]
    pub source_kind: String,
    pub pane_id: String,
    pub session_id: String,
    pub session_name: String,
    pub window_id: String,
    pub window_name: String,
    pub title: String,
    pub navigation_title: String,
    pub harness: Option<String>,
    pub harness_label: Option<String>,
    pub status: String,
    pub status_label: String,
    pub status_source: Option<String>,
    pub integration_mode: Option<String>,
    pub is_agent: bool,
    pub current_command: Option<String>,
    pub runtime_command: Option<String>,
    pub working_dir: Option<String>,
    pub linked_repository: Option<String>,
    pub workspace_name: Option<String>,
    pub preview: String,
    pub preview_provenance: String,
    pub activity_score: u64,
    pub status_rank: u8,
    pub last_activity_unix_ms: Option<u64>,
    pub last_status_change_unix_ms: Option<u64>,
    pub active_run_count: Option<u32>,
    pub pull_request: Option<ControlPullRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension_cards: Vec<ControlExtensionCard>,
}

impl AgentEntry {
    #[cfg(test)]
    pub fn test_entry(pane_id: &str) -> Self {
        let source_pane_id = SourcePaneId::new(SourceId::local(), pane_id).stable_id();
        Self {
            id: source_pane_id.clone(),
            source_pane_id,
            source_id: crate::sources::LOCAL_SOURCE_ID.to_string(),
            source_label: crate::sources::LOCAL_SOURCE_LABEL.to_string(),
            source_display_label: crate::sources::LOCAL_SOURCE_LABEL.to_string(),
            source_show_label: false,
            source_kind: "local".to_string(),
            pane_id: pane_id.to_string(),
            session_id: "$0".to_string(),
            session_name: "test".to_string(),
            window_id: "@0".to_string(),
            window_name: "test".to_string(),
            title: "test".to_string(),
            navigation_title: "test".to_string(),
            harness: None,
            harness_label: None,
            status: "unknown".to_string(),
            status_label: "UNKNOWN".to_string(),
            status_source: None,
            integration_mode: None,
            is_agent: false,
            current_command: None,
            runtime_command: None,
            working_dir: None,
            linked_repository: None,
            workspace_name: None,
            preview: String::new(),
            preview_provenance: "captured".to_string(),
            activity_score: 0,
            status_rank: 4,
            last_activity_unix_ms: None,
            last_status_change_unix_ms: None,
            active_run_count: None,
            pull_request: None,
            extension_cards: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlPullRequest {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub repository: String,
    pub branch: String,
    pub base_branch: String,
    pub author: String,
    pub status: String,
    pub status_label: String,
}

impl From<&PullRequestData> for ControlPullRequest {
    fn from(pull_request: &PullRequestData) -> Self {
        Self {
            number: pull_request.number,
            title: pull_request.title.clone(),
            url: pull_request.url.clone(),
            repository: pull_request.repository.clone(),
            branch: pull_request.branch.clone(),
            base_branch: pull_request.base_branch.clone(),
            author: pull_request.author.clone(),
            status: pull_request_status_slug(pull_request.status).to_string(),
            status_label: pull_request.status.label().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlDiagnostic {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub schema_version: u16,
    pub ok: bool,
    pub action: String,
    #[serde(default = "default_source_id")]
    pub source_id: String,
    pub pane_id: String,
    #[serde(default = "default_source_pane_id")]
    pub source_pane_id: String,
    pub bytes_sent: Option<usize>,
}

impl ActionResponse {
    pub fn new_focus(pane_id: &str) -> Self {
        focus_response(pane_id)
    }

    pub fn wrap_source(&mut self, descriptor: &SourceDescriptor) {
        self.source_id = descriptor.id.clone();
        self.source_pane_id =
            SourcePaneId::new(SourceId::new(descriptor.id.clone()), self.pane_id.clone())
                .stable_id();
    }
}

pub fn agents_response(state: &AppState, include_non_agents: bool) -> AgentsResponse {
    let summary = state.inventory_summary();
    let mut entries = Vec::new();

    for session in &state.inventory.sessions {
        for window in &session.windows {
            for pane in &window.panes {
                if !include_non_agents && !pane.is_agent() {
                    continue;
                }

                entries.push(agent_entry(
                    state,
                    pane,
                    session.id.as_str(),
                    &session.name,
                    window.id.as_str(),
                    &window.name,
                ));
            }
        }
    }

    entries.sort_by(|left, right| {
        status_rank(&left.status)
            .cmp(&status_rank(&right.status))
            .then_with(|| right.activity_score.cmp(&left.activity_score))
            .then_with(|| left.session_name.cmp(&right.session_name))
            .then_with(|| left.window_name.cmp(&right.window_name))
            .then_with(|| left.navigation_title.cmp(&right.navigation_title))
    });

    let mut diagnostics = Vec::new();
    if let Some(error) = &summary.startup_error {
        diagnostics.push(ControlDiagnostic {
            level: "error".to_string(),
            message: error.clone(),
        });
    }
    if let Some(alert) = &state.operator_alert {
        diagnostics.push(ControlDiagnostic {
            level: alert.level.label().to_ascii_lowercase(),
            message: alert.message.clone(),
        });
    }

    AgentsResponse {
        schema_version: CONTROL_API_SCHEMA_VERSION,
        generated_at_unix_ms: generated_at_unix_ms(),
        inventory: ControlInventorySummary {
            total_sessions: summary.total_sessions,
            total_windows: summary.total_windows,
            total_panes: summary.total_panes,
            visible_sessions: summary.visible_sessions,
            visible_windows: summary.visible_windows,
            visible_panes: summary.visible_panes,
        },
        entries,
        diagnostics,
        sources: Vec::new(),
        source_diagnostics: Vec::new(),
        partial_failure_count: 0,
    }
}

pub fn focus_response(pane_id: &str) -> ActionResponse {
    let source_pane_id = SourcePaneId::new(SourceId::local(), pane_id).stable_id();
    ActionResponse {
        schema_version: CONTROL_API_SCHEMA_VERSION,
        ok: true,
        action: "focus".to_string(),
        source_id: crate::sources::LOCAL_SOURCE_ID.to_string(),
        pane_id: pane_id.to_string(),
        source_pane_id,
        bytes_sent: None,
    }
}

pub fn send_response(pane_id: &str, bytes_sent: usize) -> ActionResponse {
    let source_pane_id = SourcePaneId::new(SourceId::local(), pane_id).stable_id();
    ActionResponse {
        schema_version: CONTROL_API_SCHEMA_VERSION,
        ok: true,
        action: "send".to_string(),
        source_id: crate::sources::LOCAL_SOURCE_ID.to_string(),
        pane_id: pane_id.to_string(),
        source_pane_id,
        bytes_sent: Some(bytes_sent),
    }
}

fn agent_entry(
    state: &AppState,
    pane: &Pane,
    session_id: &str,
    session_name: &str,
    window_id: &str,
    window_name: &str,
) -> AgentEntry {
    let agent = pane.agent.as_ref();
    let status = agent
        .map(|agent| status_slug(agent.status))
        .unwrap_or("unknown");
    let harness = agent.map(|agent| harness_slug(agent.harness).to_string());
    let harness_label = agent.map(|agent| agent.harness.label().to_string());
    let integration_mode = agent.map(|agent| integration_slug(agent.integration_mode).to_string());
    let status_source = agent.map(|agent| agent.integration_mode.source_label().to_string());
    let working_dir = pane.working_dir.as_ref().map(|path| path_string(path));
    let linked_repository = state
        .linked_repository_for_pane(pane)
        .map(|path| path_string(path));

    let pane_id = pane.id.as_str().to_string();
    let source_id = SourceId::new(pane.source.id.as_str().to_string());
    let source_pane_id = SourcePaneId::new(source_id, pane_id.clone()).stable_id();
    AgentEntry {
        id: source_pane_id.clone(),
        source_pane_id,
        source_id: pane.source.id.as_str().to_string(),
        source_label: pane.source.label.clone(),
        source_display_label: pane.source.label.clone(),
        source_show_label: pane.source.show_label,
        source_kind: pane.source.kind.clone(),
        pane_id,
        session_id: session_id.to_string(),
        session_name: session_name.to_string(),
        window_id: window_id.to_string(),
        window_name: window_name.to_string(),
        title: pane.title.clone(),
        navigation_title: pane.navigation_title(),
        harness,
        harness_label,
        status: status.to_string(),
        status_label: agent
            .map(|agent| agent.status.label().to_string())
            .unwrap_or_else(|| AgentStatus::Unknown.label().to_string()),
        status_source,
        integration_mode,
        is_agent: pane.is_agent(),
        current_command: pane.current_command.clone(),
        runtime_command: pane.runtime_command.clone(),
        working_dir,
        linked_repository,
        workspace_name: pane.workspace_name(),
        preview: truncate_preview(&pane.preview),
        preview_provenance: preview_provenance_slug(pane.preview_provenance).to_string(),
        activity_score: pane.recent_activity(),
        status_rank: status_rank(status),
        last_activity_unix_ms: pane.activity_unix_millis,
        last_status_change_unix_ms: agent.and_then(|agent| agent.last_status_change_unix_millis),
        active_run_count: agent.and_then(|agent| agent.active_run_count),
        pull_request: None,
        extension_cards: Vec::new(),
    }
}

pub fn attach_extension_cards(entry: &mut AgentEntry, cards: Vec<ControlExtensionCard>) {
    entry.extension_cards = cards;
}

pub fn attach_pull_request(entry: &mut AgentEntry, pull_request: &PullRequestData) {
    entry.pull_request = Some(ControlPullRequest::from(pull_request));
}

fn status_slug(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "working",
        AgentStatus::NeedsAttention => "needs-attention",
        AgentStatus::Idle => "idle",
        AgentStatus::Error => "error",
        AgentStatus::Unknown => "unknown",
    }
}

fn status_rank(status: &str) -> u8 {
    match status {
        "error" => 0,
        "needs-attention" => 1,
        "working" => 2,
        "idle" => 3,
        _ => 4,
    }
}

fn pull_request_status_slug(status: PullRequestStatus) -> &'static str {
    match status {
        PullRequestStatus::Open => "open",
        PullRequestStatus::Draft => "draft",
        PullRequestStatus::Closed => "closed",
        PullRequestStatus::Merged => "merged",
    }
}

fn harness_slug(harness: HarnessKind) -> &'static str {
    match harness {
        HarnessKind::ClaudeCode => "claude-code",
        HarnessKind::CodexCli => "codex-cli",
        HarnessKind::Pi => "pi",
        HarnessKind::GeminiCli => "gemini-cli",
        HarnessKind::OpenCode => "opencode",
    }
}

fn integration_slug(mode: IntegrationMode) -> &'static str {
    match mode {
        IntegrationMode::Native => "native",
        IntegrationMode::Compatibility => "compatibility",
    }
}

fn preview_provenance_slug(provenance: PreviewProvenance) -> &'static str {
    match provenance {
        PreviewProvenance::Captured => "captured",
        PreviewProvenance::PendingCapture => "pending-capture",
        PreviewProvenance::ReusedCached => "reused-cached",
        PreviewProvenance::CaptureFailed => "capture-failed",
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn truncate_preview(preview: &str) -> String {
    let mut truncated = preview.chars().take(MAX_PREVIEW_CHARS).collect::<String>();
    if preview.chars().count() > MAX_PREVIEW_CHARS {
        truncated.push_str("\n… truncated …");
    }
    truncated
}

fn generated_at_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{inventory, HarnessKind, PaneBuilder, SessionBuilder, WindowBuilder};
    use std::path::PathBuf;

    #[test]
    fn agents_response_defaults_to_agent_panes_only() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("main")
                .pane(
                    PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                        .working_dir("/tmp/alpha")
                        .preview("ready"),
                )
                .pane(PaneBuilder::new("%2").preview("shell")),
        )]);
        let state = AppState::with_inventory(inventory);

        let response = agents_response(&state, false);

        assert_eq!(response.schema_version, CONTROL_API_SCHEMA_VERSION);
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].pane_id, "%1");
        assert_eq!(response.entries[0].harness.as_deref(), Some("claude-code"));
        assert_eq!(
            response.entries[0].working_dir.as_deref(),
            Some("/tmp/alpha")
        );
    }

    #[test]
    fn agents_response_can_include_non_agent_panes() {
        let inventory = inventory([SessionBuilder::new("alpha")
            .window(WindowBuilder::new("main").pane(PaneBuilder::new("%2")))]);
        let state = AppState::with_inventory(inventory);

        let response = agents_response(&state, true);

        assert_eq!(response.entries.len(), 1);
        assert!(!response.entries[0].is_agent);
    }

    #[test]
    fn agents_response_includes_explicit_linked_repository() {
        let inventory = inventory([SessionBuilder::new("notes").window(
            WindowBuilder::new("main").pane(
                PaneBuilder::agent("%82", HarnessKind::Pi)
                    .working_dir("/tmp/notes")
                    .preview("working from notes"),
            ),
        )]);
        let mut state = AppState::with_inventory(inventory);
        state
            .linked_repositories
            .insert(crate::app::PaneId::new("%82"), PathBuf::from("/tmp/code"));
        state.linked_repository_pane_working_dirs.insert(
            crate::app::PaneKey::local(crate::app::PaneId::new("%82")),
            Some(PathBuf::from("/tmp/notes")),
        );

        let response = agents_response(&state, false);

        assert_eq!(response.entries.len(), 1);
        assert_eq!(
            response.entries[0].working_dir.as_deref(),
            Some("/tmp/notes")
        );
        assert_eq!(
            response.entries[0].linked_repository.as_deref(),
            Some("/tmp/code")
        );
    }
}
