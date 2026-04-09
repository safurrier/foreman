use super::{status_from_hints, CompatibilityObservation, StatusHints};
use crate::app::{AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory, PaneId};
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

const RECOGNITION_TOKENS: &[&str] = &["claude", "claude code"];
const STATUS_HINTS: StatusHints = StatusHints {
    attention: &[
        "waiting for your input",
        "waiting on you",
        "needs attention",
        "approve",
        "approval",
        "press enter to continue",
        "confirm to continue",
    ],
    error: &["error", "failed", "panic", "traceback", "exception"],
    working: &[
        "thinking",
        "working",
        "applying patch",
        "running",
        "searching",
        "reading",
        "writing",
        "editing",
        "analyzing",
        "updating",
    ],
    idle: &["ready", "idle", "awaiting task", "waiting for task", "done"],
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeNativeSignal {
    pub status: AgentStatus,
    pub activity_score: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaudeNativeOverlaySummary {
    pub applied: usize,
    pub fallback_to_compatibility: usize,
    pub warnings: Vec<String>,
}

pub trait ClaudeNativeSignalSource {
    fn signal_for_pane(
        &self,
        pane_id: &PaneId,
    ) -> Result<Option<ClaudeNativeSignal>, ClaudeNativeSignalError>;
}

#[derive(Debug)]
pub enum ClaudeNativeSignalError {
    Io(io::Error),
    Parse(serde_json::Error),
    InvalidStatus(String),
}

impl fmt::Display for ClaudeNativeSignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
            Self::InvalidStatus(status) => write!(f, "invalid Claude native status: {status}"),
        }
    }
}

impl std::error::Error for ClaudeNativeSignalError {}

impl From<io::Error> for ClaudeNativeSignalError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ClaudeNativeSignalError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(error)
    }
}

#[derive(Debug, Clone)]
pub struct FileClaudeNativeSignalSource {
    dir: PathBuf,
}

impl FileClaudeNativeSignalSource {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn signal_path(&self, pane_id: &PaneId) -> PathBuf {
        self.dir.join(format!("{}.json", pane_id.as_str()))
    }
}

impl ClaudeNativeSignalSource for FileClaudeNativeSignalSource {
    fn signal_for_pane(
        &self,
        pane_id: &PaneId,
    ) -> Result<Option<ClaudeNativeSignal>, ClaudeNativeSignalError> {
        let path = self.signal_path(pane_id);
        if !path.exists() {
            return Ok(None);
        }

        let payload = fs::read_to_string(path)?;
        let raw: RawClaudeNativeSignal = serde_json::from_str(&payload)?;
        let status = parse_native_status(&raw.status)?;
        Ok(Some(ClaudeNativeSignal {
            status,
            activity_score: raw
                .activity_score
                .unwrap_or_else(|| native_activity_score(status)),
        }))
    }
}

#[derive(Debug, Deserialize)]
struct RawClaudeNativeSignal {
    status: String,
    activity_score: Option<u64>,
}

pub(crate) fn recognition_tokens() -> &'static [&'static str] {
    RECOGNITION_TOKENS
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_hints(observation, STATUS_HINTS)
}

pub fn apply_native_signals<S: ClaudeNativeSignalSource>(
    inventory: &mut Inventory,
    source: &S,
) -> ClaudeNativeOverlaySummary {
    let mut summary = ClaudeNativeOverlaySummary::default();

    for session in &mut inventory.sessions {
        for window in &mut session.windows {
            for pane in &mut window.panes {
                let fallback_candidate = matches!(
                    pane.agent.as_ref(),
                    Some(agent)
                        if agent.harness == HarnessKind::ClaudeCode
                            && agent.integration_mode == IntegrationMode::Compatibility
                );

                match source.signal_for_pane(&pane.id) {
                    Ok(Some(signal)) => {
                        pane.agent = Some(AgentSnapshot {
                            harness: HarnessKind::ClaudeCode,
                            status: signal.status,
                            observed_status: signal.status,
                            integration_mode: IntegrationMode::Native,
                            activity_score: signal.activity_score,
                            debounce_ticks: 0,
                        });
                        summary.applied += 1;
                    }
                    Ok(None) => {
                        if fallback_candidate {
                            summary.fallback_to_compatibility += 1;
                        }
                    }
                    Err(error) => {
                        if fallback_candidate {
                            summary.fallback_to_compatibility += 1;
                        }
                        summary
                            .warnings
                            .push(format!("pane={} {error}", pane.id.as_str()));
                    }
                }
            }
        }
    }

    summary
}

fn parse_native_status(status: &str) -> Result<AgentStatus, ClaudeNativeSignalError> {
    match status.trim().to_ascii_lowercase().as_str() {
        "working" => Ok(AgentStatus::Working),
        "needs_attention" | "needs-attention" | "attention" => Ok(AgentStatus::NeedsAttention),
        "idle" => Ok(AgentStatus::Idle),
        "error" => Ok(AgentStatus::Error),
        "unknown" => Ok(AgentStatus::Unknown),
        other => Err(ClaudeNativeSignalError::InvalidStatus(other.to_string())),
    }
}

fn native_activity_score(status: AgentStatus) -> u64 {
    match status {
        AgentStatus::Working => 120,
        AgentStatus::NeedsAttention => 90,
        AgentStatus::Error => 70,
        AgentStatus::Idle => 40,
        AgentStatus::Unknown => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_native_signals, compatibility_status, FileClaudeNativeSignalSource};
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, PaneBuilder, SessionBuilder,
        WindowBuilder,
    };
    use crate::integrations::CompatibilityObservation;
    use tempfile::tempdir;

    fn observation<'a>(
        current_command: Option<&'a str>,
        title: &'a str,
        preview: &'a str,
    ) -> CompatibilityObservation<'a> {
        CompatibilityObservation::new(current_command, title, preview)
    }

    #[test]
    fn compatibility_status_maps_claude_phrases() {
        assert_eq!(
            compatibility_status(observation(Some("claude"), "shell", "Claude is thinking")),
            AgentStatus::Working
        );
        assert_eq!(
            compatibility_status(observation(
                Some("claude"),
                "shell",
                "Claude waiting for your input",
            )),
            AgentStatus::NeedsAttention
        );
        assert_eq!(
            compatibility_status(observation(Some("claude"), "shell", "Claude ready")),
            AgentStatus::Idle
        );
    }

    #[test]
    fn file_native_source_applies_precedence_over_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%1.json"),
            r#"{"status":"needs_attention","activity_score":91}"#,
        )
        .expect("signal file should exist");

        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FileClaudeNativeSignalSource::new(temp_dir.path().to_path_buf()),
        );

        let agent = inventory
            .pane(&"%1".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::NeedsAttention);
        assert_eq!(agent.activity_score, 91);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn missing_native_signal_falls_back_to_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FileClaudeNativeSignalSource::new(temp_dir.path().to_path_buf()),
        );

        let agent = inventory
            .pane(&"%1".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(agent.status, AgentStatus::Working);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.fallback_to_compatibility, 1);
    }
}
