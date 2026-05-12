use super::native::{
    signal_for_status, write_signal_file as write_native_signal_file, NativeSignal,
    NativeSignalWriteError,
};
use crate::app::AgentStatus;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const LOCK_RETRY_SLEEP: Duration = Duration::from_millis(10);
const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const STALE_LOCK_AFTER: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiHookBridgeRequest {
    pub native_dir: PathBuf,
    pub pane_id: String,
    pub run_id: Option<String>,
    pub process_id: Option<String>,
}

impl PiHookBridgeRequest {
    pub fn new(native_dir: PathBuf, pane_id: impl Into<String>) -> Self {
        Self {
            native_dir,
            pane_id: pane_id.into(),
            run_id: None,
            process_id: None,
        }
    }

    pub fn with_run_id(mut self, run_id: impl Into<String>) -> Self {
        self.run_id = Some(run_id.into());
        self
    }

    pub fn with_process_id(mut self, process_id: impl Into<String>) -> Self {
        self.process_id = Some(process_id.into());
        self
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
    Io(std::io::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for PiHookBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPaneId => write!(f, "missing pane id; set TMUX_PANE or pass --pane-id"),
            Self::Write(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "{error}"),
            Self::Serialize(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for PiHookBridgeError {}

impl From<NativeSignalWriteError> for PiHookBridgeError {
    fn from(error: NativeSignalWriteError) -> Self {
        Self::Write(error)
    }
}

impl From<std::io::Error> for PiHookBridgeError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PiHookBridgeError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

pub fn bridge_pi_event(
    request: &PiHookBridgeRequest,
    event: PiHookEventKind,
) -> Result<NativeSignal, PiHookBridgeError> {
    let status = if request.run_id.is_some() || request.process_id.is_some() {
        active_accounted_status(request, event)?
    } else {
        legacy_status_for_event(event)
    };
    let signal = signal_for_status(status);
    write_native_signal_file(&request.native_dir, request.pane_id.as_str(), &signal)?;
    Ok(signal)
}

fn legacy_status_for_event(event: PiHookEventKind) -> AgentStatus {
    match event {
        PiHookEventKind::BeforeAgentStart | PiHookEventKind::AgentStart => AgentStatus::Working,
        PiHookEventKind::AgentEnd | PiHookEventKind::SessionShutdown => AgentStatus::Idle,
    }
}

fn active_accounted_status(
    request: &PiHookBridgeRequest,
    event: PiHookEventKind,
) -> Result<AgentStatus, PiHookBridgeError> {
    fs::create_dir_all(&request.native_dir)?;
    let _guard = ActiveStateLock::acquire(&request.native_dir)?;
    let state_path = active_state_path(&request.native_dir, &request.pane_id);
    let mut state = read_active_state(&state_path)?;
    let now = unix_millis();

    match event {
        PiHookEventKind::BeforeAgentStart | PiHookEventKind::AgentStart => {
            if let Some(run_id) = request.run_id.as_ref() {
                state.runs.insert(
                    run_id.clone(),
                    ActiveRun {
                        process_id: request.process_id.clone(),
                        updated_at_unix_ms: now,
                    },
                );
            }
        }
        PiHookEventKind::AgentEnd => {
            if let Some(run_id) = request.run_id.as_ref() {
                state.runs.remove(run_id);
            }
        }
        PiHookEventKind::SessionShutdown => {
            if let Some(process_id) = request.process_id.as_ref() {
                state
                    .runs
                    .retain(|_, run| run.process_id.as_deref() != Some(process_id));
            } else if let Some(run_id) = request.run_id.as_ref() {
                state.runs.remove(run_id);
            } else {
                state.runs.clear();
            }
        }
    }

    write_active_state(&state_path, &state)?;
    Ok(if state.runs.is_empty() {
        AgentStatus::Idle
    } else {
        AgentStatus::Working
    })
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ActivePaneState {
    #[serde(default)]
    runs: BTreeMap<String, ActiveRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActiveRun {
    process_id: Option<String>,
    updated_at_unix_ms: u64,
}

fn read_active_state(path: &Path) -> Result<ActivePaneState, PiHookBridgeError> {
    if !path.exists() {
        return Ok(ActivePaneState::default());
    }
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn write_active_state(path: &Path, state: &ActivePaneState) -> Result<(), PiHookBridgeError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temp_path = path.with_extension(format!("{}.tmp", unix_millis()));
    fs::write(&temp_path, serde_json::to_vec(state)?)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn active_state_path(native_dir: &Path, pane_id: &str) -> PathBuf {
    native_dir
        .join(".pi-active")
        .join(format!("{}.json", pane_id.replace('/', "_")))
}

struct ActiveStateLock {
    path: PathBuf,
}

impl ActiveStateLock {
    fn acquire(native_dir: &Path) -> Result<Self, PiHookBridgeError> {
        let path = native_dir.join(".pi-active.lock");
        let started = SystemTime::now();
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    remove_stale_lock(&path)?;
                    if started.elapsed().unwrap_or_default() >= LOCK_TIMEOUT {
                        return Err(PiHookBridgeError::Io(error));
                    }
                    thread::sleep(LOCK_RETRY_SLEEP);
                }
                Err(error) => return Err(PiHookBridgeError::Io(error)),
            }
        }
    }
}

impl Drop for ActiveStateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn remove_stale_lock(path: &Path) -> Result<(), PiHookBridgeError> {
    let Ok(metadata) = fs::metadata(path) else {
        return Ok(());
    };
    let stale = metadata
        .modified()
        .ok()
        .and_then(|modified| modified.elapsed().ok())
        .is_some_and(|elapsed| elapsed >= STALE_LOCK_AFTER);
    if stale {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
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

    #[test]
    fn overlapping_pi_runs_keep_pane_working_until_all_complete() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let base = || PiHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%11");
        let parent = base().with_run_id("parent").with_process_id("100");
        let child_a = base().with_run_id("child-a").with_process_id("101");
        let child_b = base().with_run_id("child-b").with_process_id("102");

        bridge_pi_event(&parent, PiHookEventKind::AgentStart).expect("parent starts");
        bridge_pi_event(&child_a, PiHookEventKind::AgentStart).expect("child a starts");
        bridge_pi_event(&child_b, PiHookEventKind::AgentStart).expect("child b starts");
        let signal = bridge_pi_event(&child_a, PiHookEventKind::AgentEnd).expect("child a ends");
        assert_eq!(signal.status, crate::app::AgentStatus::Working);

        let signal = bridge_pi_event(&child_b, PiHookEventKind::AgentEnd).expect("child b ends");
        assert_eq!(signal.status, crate::app::AgentStatus::Working);

        let signal = bridge_pi_event(&parent, PiHookEventKind::AgentEnd).expect("parent ends");
        assert_eq!(signal.status, crate::app::AgentStatus::Idle);
    }

    #[test]
    fn session_shutdown_clears_runs_for_process_only() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let base = || PiHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%11");
        let first = base().with_run_id("first").with_process_id("100");
        let second = base().with_run_id("second").with_process_id("200");

        bridge_pi_event(&first, PiHookEventKind::AgentStart).expect("first starts");
        bridge_pi_event(&second, PiHookEventKind::AgentStart).expect("second starts");
        let signal = bridge_pi_event(
            &base().with_process_id("100"),
            PiHookEventKind::SessionShutdown,
        )
        .expect("first process shuts down");

        assert_eq!(signal.status, crate::app::AgentStatus::Working);
    }
}
