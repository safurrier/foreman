use crate::app::{
    Inventory, Pane, PaneId, PreviewProvenance, Session, SessionId, Window, WindowId,
};
use crate::integrations::{compatibility_snapshot, CompatibilityObservation};
use std::collections::BTreeSet;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const LIST_PANES_FORMAT: &str = "#{session_id}\t#{session_name}\t#{window_id}\t#{window_name}\t#{pane_id}\t#{pane_title}\t#{pane_current_command}\t#{pane_current_path}";

#[derive(Debug)]
pub enum TmuxError {
    Io(std::io::Error),
    Unavailable(String),
    CommandFailed { command: String, stderr: String },
    Parse(String),
}

impl fmt::Display for TmuxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Unavailable(message) => write!(f, "{message}"),
            Self::CommandFailed { command, stderr } => write!(f, "{command}: {stderr}"),
            Self::Parse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for TmuxError {}

impl From<std::io::Error> for TmuxError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmuxPaneRecord {
    pub session_id: SessionId,
    pub session_name: String,
    pub window_id: WindowId,
    pub window_name: String,
    pub pane_id: PaneId,
    pub pane_title: String,
    pub current_command: Option<String>,
    pub current_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendInputResult {
    pub pane_id: PaneId,
    pub bytes_sent: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameWindowResult {
    pub window_id: WindowId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpawnWindowResult {
    pub session_id: SessionId,
    pub window_id: WindowId,
    pub pane_id: PaneId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillPaneResult {
    pub pane_id: PaneId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventoryLoadMetrics {
    pub pane_records: usize,
    pub capture_count: usize,
    pub priority_capture_count: usize,
    pub deferred_capture_count: usize,
    pub forced_capture_count: usize,
    pub capture_failures: usize,
    pub capture_failure_panes: Vec<PaneId>,
    pub reused_preview_count: usize,
    pub list_panes_ms: u128,
    pub capture_total_ms: u128,
    pub capture_max_ms: u128,
    pub total_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventoryLoadPolicy {
    pub priority_panes: BTreeSet<PaneId>,
    pub deferred_panes: BTreeSet<PaneId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaptureReason {
    Priority,
    Deferred,
    Forced,
}

impl CaptureReason {
    fn track(self, metrics: &mut InventoryLoadMetrics) {
        metrics.capture_count += 1;
        match self {
            Self::Priority => metrics.priority_capture_count += 1,
            Self::Deferred => metrics.deferred_capture_count += 1,
            Self::Forced => metrics.forced_capture_count += 1,
        }
    }
}

pub trait TmuxBackend {
    fn list_panes(&self) -> Result<Vec<TmuxPaneRecord>, TmuxError>;

    fn capture_pane(&self, pane_id: &PaneId, lines: usize) -> Result<String, TmuxError>;

    fn focus_pane(&self, pane_id: &PaneId) -> Result<(), TmuxError>;

    fn send_input(&self, pane_id: &PaneId, text: &str) -> Result<SendInputResult, TmuxError>;

    fn rename_window(
        &self,
        window_id: &WindowId,
        name: &str,
    ) -> Result<RenameWindowResult, TmuxError>;

    fn spawn_window(
        &self,
        session_id: &SessionId,
        command: &str,
    ) -> Result<SpawnWindowResult, TmuxError>;

    fn kill_pane(&self, pane_id: &PaneId) -> Result<KillPaneResult, TmuxError>;
}

#[derive(Debug, Clone, Default)]
pub struct TmuxAdapter<B> {
    backend: B,
}

impl<B> TmuxAdapter<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: TmuxBackend> TmuxAdapter<B> {
    pub fn load_inventory(&self, capture_lines: usize) -> Result<Inventory, TmuxError> {
        self.load_inventory_profiled(capture_lines)
            .map(|(inventory, _)| inventory)
    }

    pub fn load_inventory_profiled(
        &self,
        capture_lines: usize,
    ) -> Result<(Inventory, InventoryLoadMetrics), TmuxError> {
        self.load_inventory_profiled_with_policy(
            capture_lines,
            None,
            &InventoryLoadPolicy::default(),
        )
    }

    pub fn load_inventory_profiled_with_policy(
        &self,
        capture_lines: usize,
        previous_inventory: Option<&Inventory>,
        policy: &InventoryLoadPolicy,
    ) -> Result<(Inventory, InventoryLoadMetrics), TmuxError> {
        let load_started = Instant::now();
        let list_panes_started = Instant::now();
        let pane_records = self.backend.list_panes()?;
        let list_panes_ms = list_panes_started.elapsed().as_millis();
        let mut metrics = InventoryLoadMetrics {
            pane_records: pane_records.len(),
            list_panes_ms,
            ..InventoryLoadMetrics::default()
        };
        let mut sessions: Vec<Session> = Vec::new();

        for record in pane_records {
            let previous_pane =
                previous_inventory.and_then(|inventory| inventory.pane(&record.pane_id));
            let (preview, preview_provenance) =
                match preview_resolution(previous_pane, &record, policy) {
                    Some(reason) => {
                        reason.track(&mut metrics);
                        let capture_started = Instant::now();
                        let capture =
                            match self.backend.capture_pane(&record.pane_id, capture_lines) {
                                Ok(preview) => (preview, PreviewProvenance::Captured),
                                Err(_) => {
                                    metrics.capture_failures += 1;
                                    metrics.capture_failure_panes.push(record.pane_id.clone());
                                    (String::new(), PreviewProvenance::CaptureFailed)
                                }
                            };
                        let capture_ms = capture_started.elapsed().as_millis();
                        metrics.capture_total_ms += capture_ms;
                        metrics.capture_max_ms = metrics.capture_max_ms.max(capture_ms);
                        capture
                    }
                    None => {
                        metrics.reused_preview_count += 1;
                        (
                            previous_pane
                                .map(|pane| pane.preview.clone())
                                .unwrap_or_default(),
                            PreviewProvenance::ReusedCached,
                        )
                    }
                };
            let pane_title = record.pane_title.clone();
            let current_command = record.current_command.clone();
            let agent = compatibility_snapshot(CompatibilityObservation::new(
                current_command.as_deref(),
                &pane_title,
                &preview,
            ));
            let pane = Pane {
                id: record.pane_id.clone(),
                title: non_empty(pane_title, record.pane_id.as_str()),
                current_command,
                working_dir: record.current_path.clone(),
                preview,
                preview_provenance,
                agent,
            };

            let session_index = sessions
                .iter()
                .position(|session| session.id == record.session_id);
            match session_index {
                Some(session_index) => {
                    let session = &mut sessions[session_index];
                    let window_index = session
                        .windows
                        .iter()
                        .position(|window| window.id == record.window_id);
                    match window_index {
                        Some(window_index) => session.windows[window_index].panes.push(pane),
                        None => session.windows.push(Window {
                            id: record.window_id,
                            name: non_empty(record.window_name, ""),
                            panes: vec![pane],
                        }),
                    }
                }
                None => sessions.push(Session {
                    id: record.session_id,
                    name: non_empty(record.session_name, ""),
                    windows: vec![Window {
                        id: record.window_id,
                        name: non_empty(record.window_name, ""),
                        panes: vec![pane],
                    }],
                }),
            }
        }

        metrics.total_ms = load_started.elapsed().as_millis();
        Ok((Inventory::new(sessions), metrics))
    }

    pub fn focus_pane(&self, pane_id: &PaneId) -> Result<(), TmuxError> {
        self.backend.focus_pane(pane_id)
    }

    pub fn send_input(&self, pane_id: &PaneId, text: &str) -> Result<SendInputResult, TmuxError> {
        self.backend.send_input(pane_id, text)
    }

    pub fn rename_window(
        &self,
        window_id: &WindowId,
        name: &str,
    ) -> Result<RenameWindowResult, TmuxError> {
        self.backend.rename_window(window_id, name)
    }

    pub fn spawn_window(
        &self,
        session_id: &SessionId,
        command: &str,
    ) -> Result<SpawnWindowResult, TmuxError> {
        self.backend.spawn_window(session_id, command)
    }

    pub fn kill_pane(&self, pane_id: &PaneId) -> Result<KillPaneResult, TmuxError> {
        self.backend.kill_pane(pane_id)
    }
}

fn preview_resolution(
    previous_pane: Option<&Pane>,
    record: &TmuxPaneRecord,
    policy: &InventoryLoadPolicy,
) -> Option<CaptureReason> {
    let Some(previous_pane) = previous_pane else {
        return Some(CaptureReason::Forced);
    };

    if pane_metadata_changed(previous_pane, record) || previous_pane.preview.is_empty() {
        return Some(CaptureReason::Forced);
    }

    if policy.priority_panes.contains(&record.pane_id) {
        return Some(CaptureReason::Priority);
    }

    if policy.deferred_panes.contains(&record.pane_id) {
        return Some(CaptureReason::Deferred);
    }

    None
}

fn pane_metadata_changed(previous_pane: &Pane, record: &TmuxPaneRecord) -> bool {
    previous_pane.title != non_empty(record.pane_title.clone(), record.pane_id.as_str())
        || previous_pane.current_command != record.current_command
        || previous_pane.working_dir != record.current_path
}

#[derive(Debug, Clone, Default)]
pub struct SystemTmuxBackend {
    socket_path: Option<PathBuf>,
}

impl SystemTmuxBackend {
    pub fn new(socket_path: Option<PathBuf>) -> Self {
        Self { socket_path }
    }

    fn run_tmux(&self, args: &[&str]) -> Result<String, TmuxError> {
        let mut command = Command::new("tmux");
        if let Some(socket_path) = &self.socket_path {
            command.arg("-S").arg(socket_path);
        }
        command.args(args);

        let output = command.output()?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let rendered_command = render_tmux_command(self.socket_path.as_deref(), args);
        if is_unavailable_message(&stderr) {
            return Err(TmuxError::Unavailable(stderr));
        }

        Err(TmuxError::CommandFailed {
            command: rendered_command,
            stderr,
        })
    }

    fn run_tmux_with_input(&self, args: &[&str], input: &str) -> Result<String, TmuxError> {
        let mut command = Command::new("tmux");
        if let Some(socket_path) = &self.socket_path {
            command.arg("-S").arg(socket_path);
        }
        command.args(args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }

        let output = child.wait_with_output()?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let rendered_command = render_tmux_command(self.socket_path.as_deref(), args);
        if is_unavailable_message(&stderr) {
            return Err(TmuxError::Unavailable(stderr));
        }

        Err(TmuxError::CommandFailed {
            command: rendered_command,
            stderr,
        })
    }
}

impl TmuxBackend for SystemTmuxBackend {
    fn list_panes(&self) -> Result<Vec<TmuxPaneRecord>, TmuxError> {
        let output = self.run_tmux(&["list-panes", "-a", "-F", LIST_PANES_FORMAT])?;
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(parse_pane_record)
            .collect()
    }

    fn capture_pane(&self, pane_id: &PaneId, lines: usize) -> Result<String, TmuxError> {
        let start = format!("-{}", lines.max(1));
        let output = self.run_tmux(&[
            "capture-pane",
            "-p",
            "-J",
            "-t",
            pane_id.as_str(),
            "-S",
            &start,
        ])?;
        Ok(output.trim_end_matches('\n').to_string())
    }

    fn focus_pane(&self, pane_id: &PaneId) -> Result<(), TmuxError> {
        let target = self.run_tmux(&[
            "display-message",
            "-p",
            "-t",
            pane_id.as_str(),
            "#{session_id}\t#{window_id}",
        ])?;
        let (session_id, window_id) = target.trim().split_once('\t').ok_or_else(|| {
            TmuxError::Parse(format!(
                "missing session_id or window_id in focus target: {target}"
            ))
        })?;

        match self.run_tmux(&["switch-client", "-t", session_id]) {
            Ok(_) => {}
            Err(TmuxError::CommandFailed { stderr, .. })
                if stderr.contains("no current client") || stderr.contains("no client") => {}
            Err(error) => return Err(error),
        }

        self.run_tmux(&["select-window", "-t", window_id])?;
        self.run_tmux(&["select-pane", "-t", pane_id.as_str()])?;
        Ok(())
    }

    fn send_input(&self, pane_id: &PaneId, text: &str) -> Result<SendInputResult, TmuxError> {
        let buffer_name = format!("foreman-input-{}", sanitize_tmux_name(pane_id.as_str()));
        self.run_tmux_with_input(&["load-buffer", "-b", &buffer_name, "-"], text)?;
        self.run_tmux(&[
            "paste-buffer",
            "-d",
            "-b",
            &buffer_name,
            "-t",
            pane_id.as_str(),
        ])?;
        self.run_tmux(&["send-keys", "-t", pane_id.as_str(), "Enter"])?;

        Ok(SendInputResult {
            pane_id: pane_id.clone(),
            bytes_sent: text.len(),
        })
    }

    fn rename_window(
        &self,
        window_id: &WindowId,
        name: &str,
    ) -> Result<RenameWindowResult, TmuxError> {
        self.run_tmux(&["rename-window", "-t", window_id.as_str(), name])?;
        Ok(RenameWindowResult {
            window_id: window_id.clone(),
            name: name.to_string(),
        })
    }

    fn spawn_window(
        &self,
        session_id: &SessionId,
        command: &str,
    ) -> Result<SpawnWindowResult, TmuxError> {
        let output = self.run_tmux(&[
            "new-window",
            "-d",
            "-P",
            "-F",
            "#{window_id}\t#{pane_id}",
            "-t",
            session_id.as_str(),
            command,
        ])?;
        parse_spawn_result(&output, session_id)
    }

    fn kill_pane(&self, pane_id: &PaneId) -> Result<KillPaneResult, TmuxError> {
        self.run_tmux(&["kill-pane", "-t", pane_id.as_str()])?;
        Ok(KillPaneResult {
            pane_id: pane_id.clone(),
        })
    }
}

fn parse_pane_record(line: &str) -> Result<TmuxPaneRecord, TmuxError> {
    let mut fields = line.splitn(8, '\t');
    let session_id = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing session_id in pane record: {line}")))?;
    let session_name = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing session_name in pane record: {line}")))?;
    let window_id = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing window_id in pane record: {line}")))?;
    let window_name = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing window_name in pane record: {line}")))?;
    let pane_id = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing pane_id in pane record: {line}")))?;
    let pane_title = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing pane_title in pane record: {line}")))?;
    let current_command = fields.next().ok_or_else(|| {
        TmuxError::Parse(format!("missing current_command in pane record: {line}"))
    })?;
    let current_path = fields
        .next()
        .ok_or_else(|| TmuxError::Parse(format!("missing current_path in pane record: {line}")))?;

    Ok(TmuxPaneRecord {
        session_id: SessionId::new(session_id),
        session_name: session_name.to_string(),
        window_id: WindowId::new(window_id),
        window_name: window_name.to_string(),
        pane_id: PaneId::new(pane_id),
        pane_title: pane_title.to_string(),
        current_command: (!current_command.trim().is_empty()).then(|| current_command.to_string()),
        current_path: (!current_path.trim().is_empty()).then(|| PathBuf::from(current_path)),
    })
}

fn render_tmux_command(socket_path: Option<&Path>, args: &[&str]) -> String {
    let mut rendered = String::from("tmux");
    if let Some(socket_path) = socket_path {
        rendered.push_str(" -S ");
        rendered.push_str(&socket_path.display().to_string());
    }
    for arg in args {
        rendered.push(' ');
        rendered.push_str(arg);
    }
    rendered
}

fn parse_spawn_result(
    output: &str,
    session_id: &SessionId,
) -> Result<SpawnWindowResult, TmuxError> {
    let (window_id, pane_id) = output.trim().split_once('\t').ok_or_else(|| {
        TmuxError::Parse(format!(
            "missing window_id or pane_id in spawn result: {output}"
        ))
    })?;

    Ok(SpawnWindowResult {
        session_id: session_id.clone(),
        window_id: WindowId::new(window_id),
        pane_id: PaneId::new(pane_id),
    })
}

fn is_unavailable_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("failed to connect to server")
        || lower.contains("no server running")
        || lower.contains("can't find socket")
        || lower.contains("no such file or directory")
}

fn non_empty(value: String, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value
    }
}

fn sanitize_tmux_name(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        InventoryLoadPolicy, KillPaneResult, RenameWindowResult, SendInputResult,
        SpawnWindowResult, TmuxAdapter, TmuxBackend, TmuxError, TmuxPaneRecord,
    };
    use crate::app::{
        fixtures::{inventory, PaneBuilder},
        AgentStatus, AppState, PaneId, PreviewProvenance, SelectionTarget, SessionBuilder,
        SessionId, WindowBuilder, WindowId,
    };
    use std::cell::RefCell;
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::PathBuf;

    #[derive(Debug, Default)]
    struct FakeTmuxBackend {
        panes: Vec<TmuxPaneRecord>,
        captures: BTreeMap<PaneId, Result<String, String>>,
        list_error: Option<String>,
        focused_pane: Option<PaneId>,
        capture_calls: RefCell<Vec<PaneId>>,
        sent_input: RefCell<Vec<(PaneId, String)>>,
        renamed_windows: RefCell<Vec<(WindowId, String)>>,
        spawned_windows: RefCell<Vec<(SessionId, String)>>,
        killed_panes: RefCell<Vec<PaneId>>,
    }

    impl FakeTmuxBackend {
        fn with_capture(mut self, pane_id: &str, capture: Result<&str, &str>) -> Self {
            self.captures.insert(
                PaneId::new(pane_id),
                capture.map(str::to_string).map_err(str::to_string),
            );
            self
        }
    }

    impl TmuxBackend for FakeTmuxBackend {
        fn list_panes(&self) -> Result<Vec<TmuxPaneRecord>, TmuxError> {
            if let Some(message) = &self.list_error {
                return Err(TmuxError::Unavailable(message.clone()));
            }
            Ok(self.panes.clone())
        }

        fn capture_pane(&self, pane_id: &PaneId, _lines: usize) -> Result<String, TmuxError> {
            self.capture_calls.borrow_mut().push(pane_id.clone());
            match self.captures.get(pane_id) {
                Some(Ok(capture)) => Ok(capture.clone()),
                Some(Err(message)) => Err(TmuxError::Unavailable(message.clone())),
                None => Ok(String::new()),
            }
        }

        fn focus_pane(&self, pane_id: &PaneId) -> Result<(), TmuxError> {
            if self.focused_pane.as_ref() == Some(pane_id) || self.focused_pane.is_none() {
                Ok(())
            } else {
                Err(TmuxError::Unavailable("focus failed".to_string()))
            }
        }

        fn send_input(&self, pane_id: &PaneId, text: &str) -> Result<SendInputResult, TmuxError> {
            self.sent_input
                .borrow_mut()
                .push((pane_id.clone(), text.to_string()));
            Ok(SendInputResult {
                pane_id: pane_id.clone(),
                bytes_sent: text.len(),
            })
        }

        fn rename_window(
            &self,
            window_id: &WindowId,
            name: &str,
        ) -> Result<RenameWindowResult, TmuxError> {
            self.renamed_windows
                .borrow_mut()
                .push((window_id.clone(), name.to_string()));
            Ok(RenameWindowResult {
                window_id: window_id.clone(),
                name: name.to_string(),
            })
        }

        fn spawn_window(
            &self,
            session_id: &SessionId,
            command: &str,
        ) -> Result<SpawnWindowResult, TmuxError> {
            self.spawned_windows
                .borrow_mut()
                .push((session_id.clone(), command.to_string()));
            Ok(SpawnWindowResult {
                session_id: session_id.clone(),
                window_id: WindowId::new("@spawned"),
                pane_id: PaneId::new("%spawned"),
            })
        }

        fn kill_pane(&self, pane_id: &PaneId) -> Result<KillPaneResult, TmuxError> {
            self.killed_panes.borrow_mut().push(pane_id.clone());
            Ok(KillPaneResult {
                pane_id: pane_id.clone(),
            })
        }
    }

    fn pane_record(
        session_id: &str,
        session_name: &str,
        window_id: &str,
        window_name: &str,
        pane_id: &str,
        pane_title: &str,
        current_command: Option<&str>,
    ) -> TmuxPaneRecord {
        TmuxPaneRecord {
            session_id: SessionId::new(session_id),
            session_name: session_name.to_string(),
            window_id: WindowId::new(window_id),
            window_name: window_name.to_string(),
            pane_id: PaneId::new(pane_id),
            pane_title: pane_title.to_string(),
            current_command: current_command.map(str::to_string),
            current_path: Some(PathBuf::from("/workspace")),
        }
    }

    #[test]
    fn load_inventory_groups_multi_session_records_and_marks_agents() {
        let backend = FakeTmuxBackend {
            panes: vec![
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%1",
                    "alpha-main",
                    Some("claude"),
                ),
                pane_record(
                    "$2",
                    "beta",
                    "@2",
                    "agents",
                    "%2",
                    "beta-main",
                    Some("node"),
                ),
                pane_record("$3", "notes", "@3", "notes", "%3", "notes", Some("zsh")),
            ],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code ready"))
        .with_capture("%2", Ok("Codex CLI waiting"))
        .with_capture("%3", Ok("plain shell"));

        let inventory = TmuxAdapter::new(backend)
            .load_inventory(50)
            .expect("inventory should load");

        assert_eq!(inventory.session_count(), 3);
        assert_eq!(inventory.pane_count(), 3);
        assert!(inventory
            .pane(&PaneId::new("%1"))
            .expect("pane should exist")
            .agent
            .is_some());
        assert!(inventory
            .pane(&PaneId::new("%3"))
            .expect("pane should exist")
            .agent
            .is_none());
    }

    #[test]
    fn load_inventory_maps_statuses_from_capture_preview() {
        let backend = FakeTmuxBackend {
            panes: vec![
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%1",
                    "claude",
                    Some("claude"),
                ),
                pane_record("$1", "alpha", "@1", "agents", "%2", "codex", Some("node")),
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%3",
                    "gemini",
                    Some("python3"),
                ),
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%4",
                    "opencode",
                    Some("node"),
                ),
            ],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code is thinking through the patch"))
        .with_capture("%2", Ok("Codex CLI waiting for your input"))
        .with_capture("%3", Ok("Gemini CLI ready for the next task"))
        .with_capture("%4", Ok("OpenCode exception: transport failed"));

        let inventory = TmuxAdapter::new(backend)
            .load_inventory(50)
            .expect("inventory should load");

        assert_eq!(
            inventory
                .pane(&PaneId::new("%1"))
                .and_then(|pane| pane.agent.as_ref())
                .map(|agent| agent.status),
            Some(AgentStatus::Working)
        );
        assert_eq!(
            inventory
                .pane(&PaneId::new("%2"))
                .and_then(|pane| pane.agent.as_ref())
                .map(|agent| agent.status),
            Some(AgentStatus::NeedsAttention)
        );
        assert_eq!(
            inventory
                .pane(&PaneId::new("%3"))
                .and_then(|pane| pane.agent.as_ref())
                .map(|agent| agent.status),
            Some(AgentStatus::Idle)
        );
        assert_eq!(
            inventory
                .pane(&PaneId::new("%4"))
                .and_then(|pane| pane.agent.as_ref())
                .map(|agent| agent.status),
            Some(AgentStatus::Error)
        );
    }

    #[test]
    fn load_inventory_drops_shell_panes_with_stale_harness_preview() {
        let backend = FakeTmuxBackend {
            panes: vec![pane_record(
                "$1",
                "alpha",
                "@1",
                "agents",
                "%1",
                "claude",
                Some("zsh"),
            )],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code ready"));

        let inventory = TmuxAdapter::new(backend)
            .load_inventory(50)
            .expect("inventory should load");

        assert!(inventory
            .pane(&PaneId::new("%1"))
            .expect("pane should exist")
            .agent
            .is_none());
    }

    #[test]
    fn load_inventory_keeps_panes_visible_when_capture_fails() {
        let backend = FakeTmuxBackend {
            panes: vec![pane_record(
                "$1",
                "alpha",
                "@1",
                "agents",
                "%1",
                "alpha-main",
                Some("claude"),
            )],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Err("capture failed"));

        let inventory = TmuxAdapter::new(backend)
            .load_inventory(50)
            .expect("inventory should still load");
        let pane = inventory
            .pane(&PaneId::new("%1"))
            .expect("pane should exist");

        assert_eq!(pane.preview, "");
        assert!(pane.agent.is_some());
        assert_eq!(
            pane.agent.as_ref().map(|agent| agent.status),
            Some(AgentStatus::Unknown)
        );
    }

    #[test]
    fn load_inventory_profiled_tracks_capture_failures() {
        let backend = FakeTmuxBackend {
            panes: vec![
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%1",
                    "alpha-main",
                    Some("claude"),
                ),
                pane_record("$1", "alpha", "@1", "agents", "%2", "helper", Some("zsh")),
            ],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code ready"))
        .with_capture("%2", Err("capture failed"));

        let (inventory, metrics) = TmuxAdapter::new(backend)
            .load_inventory_profiled(50)
            .expect("inventory should still load");

        assert_eq!(inventory.pane_count(), 2);
        assert_eq!(metrics.pane_records, 2);
        assert_eq!(metrics.capture_count, 2);
        assert_eq!(metrics.capture_failures, 1);
        assert_eq!(metrics.capture_failure_panes, vec![PaneId::new("%2")]);
        assert!(metrics.total_ms >= metrics.list_panes_ms);
        assert!(metrics.capture_total_ms >= metrics.capture_max_ms);
        assert_eq!(
            inventory
                .pane(&PaneId::new("%2"))
                .expect("pane should exist")
                .preview_provenance,
            PreviewProvenance::CaptureFailed
        );
    }

    #[test]
    fn load_inventory_profiled_reuses_stable_offscreen_previews() {
        let previous_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("agents")
                .pane(
                    PaneBuilder::agent("%1", crate::app::HarnessKind::ClaudeCode)
                        .title("alpha-main")
                        .current_command("claude")
                        .working_dir("/workspace")
                        .preview("Claude Code old selected preview"),
                )
                .pane(
                    PaneBuilder::agent("%2", crate::app::HarnessKind::CodexCli)
                        .title("beta-main")
                        .current_command("node")
                        .working_dir("/workspace")
                        .preview("Codex CLI cached offscreen preview"),
                ),
        )]);
        let backend = FakeTmuxBackend {
            panes: vec![
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%1",
                    "alpha-main",
                    Some("claude"),
                ),
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%2",
                    "beta-main",
                    Some("node"),
                ),
            ],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code fresh selected preview"))
        .with_capture("%2", Ok("Codex CLI fresh but deferred"));

        let policy = InventoryLoadPolicy {
            priority_panes: BTreeSet::from([PaneId::new("%1")]),
            deferred_panes: BTreeSet::new(),
        };
        let (inventory, metrics) = TmuxAdapter::new(backend)
            .load_inventory_profiled_with_policy(40, Some(&previous_inventory), &policy)
            .expect("inventory should load");

        assert_eq!(metrics.capture_count, 1);
        assert_eq!(metrics.priority_capture_count, 1);
        assert_eq!(metrics.reused_preview_count, 1);
        assert_eq!(
            inventory
                .pane(&PaneId::new("%2"))
                .expect("pane should exist")
                .preview,
            "Codex CLI cached offscreen preview"
        );
        assert_eq!(
            inventory
                .pane(&PaneId::new("%2"))
                .expect("pane should exist")
                .preview_provenance,
            PreviewProvenance::ReusedCached
        );
    }

    #[test]
    fn load_inventory_profiled_forces_recapture_when_metadata_changes() {
        let previous_inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("agents").pane(
                PaneBuilder::agent("%1", crate::app::HarnessKind::ClaudeCode)
                    .title("alpha-main")
                    .current_command("claude")
                    .working_dir("/workspace")
                    .preview("Claude Code old preview"),
            ),
        )]);
        let backend = FakeTmuxBackend {
            panes: vec![pane_record(
                "$1",
                "alpha",
                "@1",
                "agents",
                "%1",
                "alpha-main",
                Some("zsh"),
            )],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("plain shell"));

        let policy = InventoryLoadPolicy::default();
        let (inventory, metrics) = TmuxAdapter::new(backend)
            .load_inventory_profiled_with_policy(40, Some(&previous_inventory), &policy)
            .expect("inventory should load");

        assert_eq!(metrics.capture_count, 1);
        assert_eq!(metrics.forced_capture_count, 1);
        assert_eq!(metrics.reused_preview_count, 0);
        let pane = inventory
            .pane(&PaneId::new("%1"))
            .expect("pane should exist");
        assert_eq!(pane.preview, "plain shell");
        assert_eq!(pane.preview_provenance, PreviewProvenance::Captured);
        assert!(pane.agent.is_none());
    }

    #[test]
    fn default_visibility_hides_non_agent_sessions_and_panes() {
        let backend = FakeTmuxBackend {
            panes: vec![
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%1",
                    "alpha-main",
                    Some("claude"),
                ),
                pane_record(
                    "$1",
                    "alpha",
                    "@1",
                    "agents",
                    "%2",
                    "alpha-helper",
                    Some("zsh"),
                ),
                pane_record("$2", "notes", "@2", "notes", "%3", "notes", Some("zsh")),
            ],
            ..FakeTmuxBackend::default()
        }
        .with_capture("%1", Ok("Claude Code ready"))
        .with_capture("%2", Ok("plain shell"))
        .with_capture("%3", Ok("plain shell"));

        let state = AppState::with_inventory(
            TmuxAdapter::new(backend)
                .load_inventory(50)
                .expect("inventory should load"),
        );

        let summary = state.inventory_summary();
        assert_eq!(summary.visible_sessions, 1);
        assert_eq!(summary.visible_panes, 1);
        assert_eq!(
            state.selection,
            Some(SelectionTarget::Session(SessionId::new("$1")))
        );
    }

    #[test]
    fn load_inventory_returns_unavailable_error_when_tmux_cannot_list_panes() {
        let backend = FakeTmuxBackend {
            list_error: Some("failed to connect to server".to_string()),
            ..FakeTmuxBackend::default()
        };

        let error = TmuxAdapter::new(backend)
            .load_inventory(50)
            .expect_err("inventory should fail");

        assert!(matches!(error, TmuxError::Unavailable(_)));
    }

    #[test]
    fn focus_pane_delegates_to_backend() {
        let adapter = TmuxAdapter::new(FakeTmuxBackend::default());

        let result = adapter.focus_pane(&PaneId::new("%1"));

        assert!(result.is_ok());
    }

    #[test]
    fn action_methods_delegate_to_backend_with_structured_results() {
        let backend = FakeTmuxBackend::default();
        let adapter = TmuxAdapter::new(backend);

        let send_result = adapter
            .send_input(&PaneId::new("%1"), "hello\nworld")
            .expect("send input should succeed");
        let rename_result = adapter
            .rename_window(&WindowId::new("@1"), "renamed")
            .expect("rename should succeed");
        let spawn_result = adapter
            .spawn_window(&SessionId::new("$1"), "claude")
            .expect("spawn should succeed");
        let kill_result = adapter
            .kill_pane(&PaneId::new("%1"))
            .expect("kill should succeed");

        assert_eq!(
            send_result,
            SendInputResult {
                pane_id: PaneId::new("%1"),
                bytes_sent: "hello\nworld".len(),
            }
        );
        assert_eq!(
            rename_result,
            RenameWindowResult {
                window_id: WindowId::new("@1"),
                name: "renamed".to_string(),
            }
        );
        assert_eq!(
            spawn_result,
            SpawnWindowResult {
                session_id: SessionId::new("$1"),
                window_id: WindowId::new("@spawned"),
                pane_id: PaneId::new("%spawned"),
            }
        );
        assert_eq!(
            kill_result,
            KillPaneResult {
                pane_id: PaneId::new("%1"),
            }
        );
    }
}
