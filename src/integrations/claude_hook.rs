use super::claude::ClaudeNativeSignal;
use crate::app::AgentStatus;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeHookBridgeRequest {
    pub native_dir: PathBuf,
    pub pane_id: String,
}

impl ClaudeHookBridgeRequest {
    pub fn new(native_dir: PathBuf, pane_id: impl Into<String>) -> Self {
        Self {
            native_dir,
            pane_id: pane_id.into(),
        }
    }

    pub fn with_tmux_pane(
        native_dir: PathBuf,
        pane_id: Option<String>,
    ) -> Result<Self, ClaudeHookBridgeError> {
        let pane_id = pane_id
            .or_else(|| std::env::var("TMUX_PANE").ok())
            .ok_or(ClaudeHookBridgeError::MissingPaneId)?;
        Ok(Self::new(native_dir, pane_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeHookEventKind {
    UserPromptSubmit,
    Notification,
    Stop,
    StopFailure,
}

#[derive(Debug)]
pub enum ClaudeHookBridgeError {
    Io(io::Error),
    Parse(serde_json::Error),
    MissingPaneId,
    Serialize(serde_json::Error),
}

impl fmt::Display for ClaudeHookBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
            Self::MissingPaneId => write!(f, "missing pane id; set TMUX_PANE or pass --pane-id"),
            Self::Serialize(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ClaudeHookBridgeError {}

impl From<io::Error> for ClaudeHookBridgeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Deserialize)]
struct RawClaudeHookEvent {
    hook_event_name: String,
    notification_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct SignalFile<'a> {
    status: &'a str,
    activity_score: u64,
}

pub fn bridge_claude_hook_input<R: Read>(
    request: &ClaudeHookBridgeRequest,
    reader: R,
) -> Result<Option<ClaudeHookEventKind>, ClaudeHookBridgeError> {
    let event: RawClaudeHookEvent =
        serde_json::from_reader(reader).map_err(ClaudeHookBridgeError::Parse)?;

    let Some((kind, signal)) = signal_for_event(&event) else {
        return Ok(None);
    };

    write_signal_file(request, &signal)?;
    Ok(Some(kind))
}

fn signal_for_event(
    event: &RawClaudeHookEvent,
) -> Option<(ClaudeHookEventKind, ClaudeNativeSignal)> {
    match event.hook_event_name.as_str() {
        "UserPromptSubmit" => Some((
            ClaudeHookEventKind::UserPromptSubmit,
            signal_for_status(AgentStatus::Working),
        )),
        "Notification" => match event.notification_type.as_deref() {
            Some("permission_prompt") | Some("elicitation_dialog") => Some((
                ClaudeHookEventKind::Notification,
                signal_for_status(AgentStatus::NeedsAttention),
            )),
            Some("idle_prompt") => Some((
                ClaudeHookEventKind::Notification,
                signal_for_status(AgentStatus::Idle),
            )),
            _ => None,
        },
        "Stop" => Some((
            ClaudeHookEventKind::Stop,
            signal_for_status(AgentStatus::Idle),
        )),
        "StopFailure" => Some((
            ClaudeHookEventKind::StopFailure,
            signal_for_status(AgentStatus::Error),
        )),
        _ => None,
    }
}

fn signal_for_status(status: AgentStatus) -> ClaudeNativeSignal {
    ClaudeNativeSignal {
        status,
        activity_score: match status {
            AgentStatus::Working => 120,
            AgentStatus::NeedsAttention => 90,
            AgentStatus::Error => 70,
            AgentStatus::Idle => 40,
            AgentStatus::Unknown => 0,
        },
    }
}

fn write_signal_file(
    request: &ClaudeHookBridgeRequest,
    signal: &ClaudeNativeSignal,
) -> Result<(), ClaudeHookBridgeError> {
    fs::create_dir_all(&request.native_dir)?;

    let final_path = request
        .native_dir
        .join(format!("{}.json", request.pane_id.as_str()));
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let temp_path = request.native_dir.join(format!(
        ".{}.{}.tmp",
        request.pane_id.as_str(),
        unique_suffix
    ));
    let payload = SignalFile {
        status: status_label(signal.status),
        activity_score: signal.activity_score,
    };
    let contents = serde_json::to_vec(&payload).map_err(ClaudeHookBridgeError::Serialize)?;
    fs::write(&temp_path, contents)?;

    if let Err(error) = fs::rename(&temp_path, &final_path) {
        if final_path.exists() {
            fs::remove_file(&final_path)?;
            fs::rename(&temp_path, &final_path)?;
        } else {
            return Err(ClaudeHookBridgeError::Io(error));
        }
    }

    Ok(())
}

fn status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "working",
        AgentStatus::NeedsAttention => "needs_attention",
        AgentStatus::Error => "error",
        AgentStatus::Idle => "idle",
        AgentStatus::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{bridge_claude_hook_input, ClaudeHookBridgeRequest, ClaudeHookEventKind};
    use tempfile::tempdir;

    fn bridge_request() -> (tempfile::TempDir, ClaudeHookBridgeRequest) {
        let temp_dir = tempdir().expect("temp dir should exist");
        let request = ClaudeHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%9");
        (temp_dir, request)
    }

    #[test]
    fn user_prompt_submit_writes_working_signal() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_claude_hook_input(
            &request,
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"Say hi"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(ClaudeHookEventKind::UserPromptSubmit));
        let contents = std::fs::read_to_string(temp_dir.path().join("%9.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
    }

    #[test]
    fn notification_permission_prompt_writes_attention_signal() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_claude_hook_input(
            &request,
            br#"{"hook_event_name":"Notification","notification_type":"permission_prompt"}"#
                .as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(ClaudeHookEventKind::Notification));
        let contents = std::fs::read_to_string(temp_dir.path().join("%9.json"))
            .expect("signal file should exist");
        assert_eq!(
            contents,
            r#"{"status":"needs_attention","activity_score":90}"#
        );
    }

    #[test]
    fn unsupported_notification_is_ignored() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_claude_hook_input(
            &request,
            br#"{"hook_event_name":"Notification","notification_type":"auth_success"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, None);
        assert!(!temp_dir.path().join("%9.json").exists());
    }

    #[test]
    fn stop_failure_writes_error_signal() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_claude_hook_input(
            &request,
            br#"{"hook_event_name":"StopFailure","error":"rate_limit"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(ClaudeHookEventKind::StopFailure));
        let contents = std::fs::read_to_string(temp_dir.path().join("%9.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"error","activity_score":70}"#);
    }
}
