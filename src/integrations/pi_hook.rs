use super::native::{
    signal_for_status, write_signal_file as write_native_signal_file, NativeSignal,
    NativeSignalWriteError,
};
use crate::app::AgentStatus;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiHookBridgeRequest {
    pub native_dir: PathBuf,
    pub pane_id: String,
}

impl PiHookBridgeRequest {
    pub fn new(native_dir: PathBuf, pane_id: impl Into<String>) -> Self {
        Self {
            native_dir,
            pane_id: pane_id.into(),
        }
    }

    pub fn with_tmux_pane(
        native_dir: PathBuf,
        pane_id: Option<String>,
    ) -> Result<Self, PiHookBridgeError> {
        let pane_id = pane_id
            .or_else(|| std::env::var("TMUX_PANE").ok())
            .ok_or(PiHookBridgeError::MissingPaneId)?;
        Ok(Self::new(native_dir, pane_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiHookEventKind {
    BeforeAgentStart,
    AgentStart,
    AgentEnd,
    SessionShutdown,
}

#[derive(Debug)]
pub enum PiHookBridgeError {
    MissingPaneId,
    Write(NativeSignalWriteError),
}

impl fmt::Display for PiHookBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPaneId => write!(f, "missing pane id; set TMUX_PANE or pass --pane-id"),
            Self::Write(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PiHookBridgeError {}

impl From<NativeSignalWriteError> for PiHookBridgeError {
    fn from(error: NativeSignalWriteError) -> Self {
        Self::Write(error)
    }
}

pub fn bridge_pi_event(
    request: &PiHookBridgeRequest,
    event: PiHookEventKind,
) -> Result<NativeSignal, PiHookBridgeError> {
    let signal = signal_for_event(event);
    write_native_signal_file(&request.native_dir, request.pane_id.as_str(), &signal)?;
    Ok(signal)
}

fn signal_for_event(event: PiHookEventKind) -> NativeSignal {
    match event {
        PiHookEventKind::BeforeAgentStart | PiHookEventKind::AgentStart => {
            signal_for_status(AgentStatus::Working)
        }
        PiHookEventKind::AgentEnd | PiHookEventKind::SessionShutdown => {
            signal_for_status(AgentStatus::Idle)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{bridge_pi_event, PiHookBridgeRequest, PiHookEventKind};
    use tempfile::tempdir;

    fn bridge_request() -> (tempfile::TempDir, PiHookBridgeRequest) {
        let temp_dir = tempdir().expect("temp dir should exist");
        let request = PiHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%11");
        (temp_dir, request)
    }

    #[test]
    fn agent_start_writes_working_signal() {
        let (temp_dir, request) = bridge_request();

        let signal =
            bridge_pi_event(&request, PiHookEventKind::AgentStart).expect("bridge should succeed");

        assert_eq!(signal.status, crate::app::AgentStatus::Working);
        let contents = std::fs::read_to_string(temp_dir.path().join("%11.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
    }

    #[test]
    fn agent_end_writes_idle_signal() {
        let (temp_dir, request) = bridge_request();

        let signal =
            bridge_pi_event(&request, PiHookEventKind::AgentEnd).expect("bridge should succeed");

        assert_eq!(signal.status, crate::app::AgentStatus::Idle);
        let contents = std::fs::read_to_string(temp_dir.path().join("%11.json"))
            .expect("signal file should exist");
        assert_eq!(contents, r#"{"status":"idle","activity_score":40}"#);
    }
}
