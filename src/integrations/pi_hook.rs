use super::native::{
    signal_for_status, write_signal_file as write_native_signal_file, NativeSignal,
    NativeSignalWriteError,
};
use super::native_events::{derive_native_state, NativeEvent, NativeEventKind};
use crate::app::AgentStatus;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System};

const LOCK_RETRY_SLEEP: Duration = Duration::from_millis(10);
const LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const STALE_LOCK_AFTER: Duration = Duration::from_secs(30);
const STALE_ACTIVE_PROCESS_AFTER: Duration = Duration::from_secs(60);

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
    if request.run_id.is_some() || request.process_id.is_some() {
        return active_accounted_signal(request, event);
    }

    let signal = signal_for_status(legacy_status_for_event(event));
    write_native_signal_file(&request.native_dir, request.pane_id.as_str(), &signal)?;
    Ok(signal)
}

fn legacy_status_for_event(event: PiHookEventKind) -> AgentStatus {
    match event {
        PiHookEventKind::BeforeAgentStart | PiHookEventKind::AgentStart => AgentStatus::Working,
        PiHookEventKind::AgentEnd | PiHookEventKind::SessionShutdown => AgentStatus::Idle,
    }
}

fn active_accounted_signal(
    request: &PiHookBridgeRequest,
    event: PiHookEventKind,
) -> Result<NativeSignal, PiHookBridgeError> {
    fs::create_dir_all(&request.native_dir)?;
    let _guard = EventMaterializationLock::acquire(&request.native_dir)?;
    let occurred_at = unix_millis();
    let native_event = event_for_request(request, event, occurred_at, unix_sequence())?;
    append_event(&request.native_dir, &native_event)?;
    let mut events = read_events(&request.native_dir, &request.pane_id)?;
    let mut state = derive_native_state(&events);

    let stale_processes = stale_dead_processes(&state, occurred_at, request.process_id.as_deref());
    for process_id in stale_processes {
        append_event(
            &request.native_dir,
            &NativeEvent::process_exited("pi", &request.pane_id, process_id, unix_millis())
                .with_sequence(unix_sequence()),
        )?;
    }
    if !state.active_processes.is_empty() {
        events = read_events(&request.native_dir, &request.pane_id)?;
        state = derive_native_state(&events);
    }

    let signal = NativeSignal {
        status: state.status,
        activity_score: signal_for_status(state.status).activity_score,
        recent_activity_unix_millis: state.last_activity_unix_ms,
        active_run_count: Some(state.active_run_count),
        last_activity_unix_millis: state.last_activity_unix_ms,
        last_status_change_unix_millis: state.last_status_change_unix_ms,
    };
    write_native_signal_file(&request.native_dir, request.pane_id.as_str(), &signal)?;
    compact_resolved_events(
        &request.native_dir,
        &request.pane_id,
        state.active_run_count,
    )?;
    Ok(signal)
}

fn event_for_request(
    request: &PiHookBridgeRequest,
    event: PiHookEventKind,
    occurred_at_unix_ms: u64,
    sequence: u64,
) -> Result<NativeEvent, PiHookBridgeError> {
    let mut native_event = match event {
        PiHookEventKind::BeforeAgentStart | PiHookEventKind::AgentStart => NativeEvent {
            schema_version: 1,
            source: "pi".to_string(),
            pane_id: request.pane_id.clone(),
            kind: NativeEventKind::RunStarted,
            occurred_at_unix_ms,
            sequence,
            run_id: request.run_id.clone(),
            process_id: request.process_id.clone(),
            status: None,
        },
        PiHookEventKind::AgentEnd => NativeEvent {
            schema_version: 1,
            source: "pi".to_string(),
            pane_id: request.pane_id.clone(),
            kind: NativeEventKind::RunFinished,
            occurred_at_unix_ms,
            sequence,
            run_id: request.run_id.clone(),
            process_id: request.process_id.clone(),
            status: None,
        },
        PiHookEventKind::SessionShutdown => NativeEvent {
            schema_version: 1,
            source: "pi".to_string(),
            pane_id: request.pane_id.clone(),
            kind: NativeEventKind::ProcessExited,
            occurred_at_unix_ms,
            sequence,
            run_id: request.run_id.clone(),
            process_id: request.process_id.clone(),
            status: None,
        },
    };

    if matches!(
        native_event.kind,
        NativeEventKind::RunStarted | NativeEventKind::RunFinished
    ) && native_event.run_id.is_none()
    {
        native_event.run_id = Some(format!(
            "{}:{}",
            native_event.source,
            native_event
                .process_id
                .as_deref()
                .unwrap_or("unknown-process")
        ));
    }

    Ok(native_event)
}

fn append_event(native_dir: &Path, event: &NativeEvent) -> Result<(), PiHookBridgeError> {
    let event_dir = event_dir(native_dir, &event.pane_id);
    fs::create_dir_all(&event_dir)?;
    let final_path = event_dir.join(format!(
        "{}-{}.json",
        event.occurred_at_unix_ms, event.sequence
    ));
    let temp_path = event_dir.join(format!(
        ".{}-{}.tmp",
        event.occurred_at_unix_ms, event.sequence
    ));
    fs::write(&temp_path, serde_json::to_vec(event)?)?;
    fs::rename(temp_path, final_path)?;
    Ok(())
}

fn read_events(native_dir: &Path, pane_id: &str) -> Result<Vec<NativeEvent>, PiHookBridgeError> {
    let event_dir = event_dir(native_dir, pane_id);
    if !event_dir.exists() {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    for entry in fs::read_dir(event_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        let contents = fs::read_to_string(path)?;
        events.push(serde_json::from_str(&contents)?);
    }
    Ok(events)
}

fn stale_dead_processes(
    state: &super::native_events::NativeDerivedState,
    now_unix_ms: u64,
    current_process_id: Option<&str>,
) -> Vec<String> {
    state
        .active_processes
        .iter()
        .filter_map(|(process_id, started_at)| {
            if current_process_id == Some(process_id.as_str()) {
                return None;
            }
            let age = Duration::from_millis(now_unix_ms.saturating_sub(*started_at));
            if age < STALE_ACTIVE_PROCESS_AFTER || process_is_running(process_id) {
                return None;
            }
            Some(process_id.clone())
        })
        .collect()
}

fn process_is_running(process_id: &str) -> bool {
    let Ok(process_id) = process_id.parse::<u32>() else {
        return true;
    };
    let system = System::new_all();
    system.process(Pid::from_u32(process_id)).is_some()
}

fn compact_resolved_events(
    native_dir: &Path,
    pane_id: &str,
    active_run_count: u32,
) -> Result<(), PiHookBridgeError> {
    if active_run_count != 0 {
        return Ok(());
    }
    match fs::remove_dir_all(event_dir(native_dir, pane_id)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(PiHookBridgeError::Io(error)),
    }
}

fn event_dir(native_dir: &Path, pane_id: &str) -> PathBuf {
    native_dir
        .join(".pi-events")
        .join(pane_id.replace('/', "_"))
}

struct EventMaterializationLock {
    path: PathBuf,
}

impl EventMaterializationLock {
    fn acquire(native_dir: &Path) -> Result<Self, PiHookBridgeError> {
        let path = native_dir.join(".pi-events.lock");
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

impl Drop for EventMaterializationLock {
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

fn unix_sequence() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{
        append_event, bridge_pi_event, event_dir, read_events, unix_millis, NativeEvent,
        PiHookBridgeRequest, PiHookEventKind, STALE_ACTIVE_PROCESS_AFTER,
    };
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

    #[test]
    fn final_run_finish_compacts_resolved_event_history() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let request = PiHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%11")
            .with_run_id("run")
            .with_process_id(std::process::id().to_string());

        bridge_pi_event(&request, PiHookEventKind::AgentStart).expect("run starts");
        assert!(event_dir(temp_dir.path(), "%11").exists());

        let signal = bridge_pi_event(&request, PiHookEventKind::AgentEnd).expect("run ends");

        assert_eq!(signal.status, crate::app::AgentStatus::Idle);
        assert_eq!(signal.active_run_count, Some(0));
        assert!(!event_dir(temp_dir.path(), "%11").exists());
    }

    #[test]
    fn stale_dead_process_runs_are_cleared_on_next_materialization() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let stale_started_at = unix_millis()
            .saturating_sub(STALE_ACTIVE_PROCESS_AFTER.as_millis() as u64)
            .saturating_sub(1_000);
        let stale_process = "99999999";
        append_event(
            temp_dir.path(),
            &NativeEvent::run_started("pi", "%11", "orphan", stale_process, stale_started_at),
        )
        .expect("stale event should be written");

        let request = PiHookBridgeRequest::new(temp_dir.path().to_path_buf(), "%11")
            .with_run_id("fresh")
            .with_process_id(std::process::id().to_string());
        let signal = bridge_pi_event(&request, PiHookEventKind::AgentStart).expect("fresh starts");

        assert_eq!(signal.status, crate::app::AgentStatus::Working);
        assert_eq!(signal.active_run_count, Some(1));
        let events = read_events(temp_dir.path(), "%11").expect("events should read");
        assert!(events.iter().any(|event| {
            event.kind == super::NativeEventKind::ProcessExited
                && event.process_id.as_deref() == Some(stale_process)
        }));
    }
}
