use crate::app::{AgentStatus, AppState, HarnessKind, IntegrationMode, Pane, PreviewProvenance};
use crate::services::pull_requests::{PullRequestData, PullRequestStatus};
use serde::Serialize;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CONTROL_API_SCHEMA_VERSION: u16 = 1;
const MAX_PREVIEW_CHARS: usize = 1200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsResponse {
    pub schema_version: u16,
    pub generated_at_unix_ms: u128,
    pub inventory: ControlInventorySummary,
    pub entries: Vec<AgentEntry>,
    pub diagnostics: Vec<ControlDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlInventorySummary {
    pub total_sessions: usize,
    pub total_windows: usize,
    pub total_panes: usize,
    pub visible_sessions: usize,
    pub visible_windows: usize,
    pub visible_panes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEntry {
    pub id: String,
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
    pub workspace_name: Option<String>,
    pub preview: String,
    pub preview_provenance: String,
    pub activity_score: u64,
    pub pull_request: Option<ControlPullRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlDiagnostic {
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub schema_version: u16,
    pub ok: bool,
    pub action: &'static str,
    pub pane_id: String,
    pub bytes_sent: Option<usize>,
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
    }
}

pub fn focus_response(pane_id: &str) -> ActionResponse {
    ActionResponse {
        schema_version: CONTROL_API_SCHEMA_VERSION,
        ok: true,
        action: "focus",
        pane_id: pane_id.to_string(),
        bytes_sent: None,
    }
}

pub fn send_response(pane_id: &str, bytes_sent: usize) -> ActionResponse {
    ActionResponse {
        schema_version: CONTROL_API_SCHEMA_VERSION,
        ok: true,
        action: "send",
        pane_id: pane_id.to_string(),
        bytes_sent: Some(bytes_sent),
    }
}

fn agent_entry(
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

    AgentEntry {
        id: format!("pane:{}", pane.id.as_str()),
        pane_id: pane.id.as_str().to_string(),
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
        workspace_name: pane.workspace_name(),
        preview: truncate_preview(&pane.preview),
        preview_provenance: preview_provenance_slug(pane.preview_provenance).to_string(),
        activity_score: pane.recent_activity(),
        pull_request: None,
    }
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
}
