use super::native::{
    signal_for_status, write_signal_file as write_native_signal_file, NativeSignal,
    NativeSignalWriteError,
};
use crate::app::AgentStatus;
use serde::Deserialize;
use std::fmt;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexHookBridgeRequest {
    pub native_dir: PathBuf,
    pub pane_id: String,
}

impl CodexHookBridgeRequest {
    pub fn new(native_dir: PathBuf, pane_id: impl Into<String>) -> Self {
        Self {
            native_dir,
            pane_id: pane_id.into(),
        }
    }

    pub fn with_tmux_pane(
        native_dir: PathBuf,
        pane_id: Option<String>,
    ) -> Result<Self, CodexHookBridgeError> {
        let pane_id = pane_id
            .or_else(|| std::env::var("TMUX_PANE").ok())
            .ok_or(CodexHookBridgeError::MissingPaneId)?;
        Ok(Self::new(native_dir, pane_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexHookEventKind {
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    Stop,
}

#[derive(Debug)]
pub enum CodexHookBridgeError {
    Parse(serde_json::Error),
    MissingPaneId,
    Write(NativeSignalWriteError),
}

impl fmt::Display for CodexHookBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => write!(f, "{error}"),
            Self::MissingPaneId => write!(f, "missing pane id; set TMUX_PANE or pass --pane-id"),
            Self::Write(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for CodexHookBridgeError {}

impl From<NativeSignalWriteError> for CodexHookBridgeError {
    fn from(error: NativeSignalWriteError) -> Self {
        Self::Write(error)
    }
}

#[derive(Debug, Deserialize)]
struct RawCodexHookEvent {
    hook_event_name: String,
}

pub fn bridge_codex_hook_input<R: Read>(
    request: &CodexHookBridgeRequest,
    reader: R,
) -> Result<Option<CodexHookEventKind>, CodexHookBridgeError> {
    let event: RawCodexHookEvent =
        serde_json::from_reader(reader).map_err(CodexHookBridgeError::Parse)?;

    let Some((kind, signal)) = signal_for_event(&event) else {
        return Ok(None);
    };

    write_native_signal_file(&request.native_dir, request.pane_id.as_str(), &signal)?;
    Ok(Some(kind))
}

fn signal_for_event(event: &RawCodexHookEvent) -> Option<(CodexHookEventKind, NativeSignal)> {
    match event.hook_event_name.as_str() {
        "UserPromptSubmit" => Some((
            CodexHookEventKind::UserPromptSubmit,
            signal_for_status(AgentStatus::Working),
        )),
        "PreToolUse" => Some((
            CodexHookEventKind::PreToolUse,
            signal_for_status(AgentStatus::Working),
        )),
        "PostToolUse" => Some((
            CodexHookEventKind::PostToolUse,
            signal_for_status(AgentStatus::Working),
        )),
        "Stop" => Some((
            CodexHookEventKind::Stop,
            signal_for_status(AgentStatus::Idle),
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{bridge_codex_hook_input, CodexHookBridgeRequest, CodexHookEventKind};
    use tempfile::tempdir;

    fn bridge_request() -> (tempfile::TempDir, CodexHookBridgeRequest) {
        let temp_dir = tempdir().expect("temp dir should exist");
        let request = CodexHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%4");
        (temp_dir, request)
    }

    #[test]
    fn user_prompt_submit_writes_working_signal() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_codex_hook_input(
            &request,
            br#"{"hook_event_name":"UserPromptSubmit","prompt":"Say hi"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(CodexHookEventKind::UserPromptSubmit));
        let contents = std::fs::read_to_string(temp_dir.path().join("%4.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
    }

    #[test]
    fn pre_tool_use_keeps_working_signal_hot() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_codex_hook_input(
            &request,
            br#"{"hook_event_name":"PreToolUse","tool_name":"Bash"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(CodexHookEventKind::PreToolUse));
        let contents = std::fs::read_to_string(temp_dir.path().join("%4.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
    }

    #[test]
    fn stop_writes_idle_signal() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_codex_hook_input(
            &request,
            br#"{"hook_event_name":"Stop","stop_hook_active":false}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, Some(CodexHookEventKind::Stop));
        let contents = std::fs::read_to_string(temp_dir.path().join("%4.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"idle","activity_score":40}"#);
    }

    #[test]
    fn unsupported_event_is_ignored() {
        let (temp_dir, request) = bridge_request();

        let event = bridge_codex_hook_input(
            &request,
            br#"{"hook_event_name":"SessionStart","source":"resume"}"#.as_slice(),
        )
        .expect("bridge should succeed");

        assert_eq!(event, None);
        assert!(!temp_dir.path().join("%4.json").exists());
    }
}
