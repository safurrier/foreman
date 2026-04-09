use crate::app::{
    AgentSnapshot, AgentStatus, IntegrationMode, Inventory, Pane, PaneId, Session, SessionId,
    Window, WindowId,
};
use crate::integrations::recognize_harness;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

const LIST_PANES_FORMAT: &str = "#{session_id}\t#{session_name}\t#{window_id}\t#{window_name}\t#{pane_id}\t#{pane_title}\t#{pane_current_command}";

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
}

pub trait TmuxBackend {
    fn list_panes(&self) -> Result<Vec<TmuxPaneRecord>, TmuxError>;

    fn capture_pane(&self, pane_id: &PaneId, lines: usize) -> Result<String, TmuxError>;
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
        let pane_records = self.backend.list_panes()?;
        let mut sessions: Vec<Session> = Vec::new();

        for record in pane_records {
            let preview = self
                .backend
                .capture_pane(&record.pane_id, capture_lines)
                .unwrap_or_default();
            let harness = recognize_harness(
                record.current_command.as_deref(),
                &record.pane_title,
                &preview,
            );

            let pane = Pane {
                id: record.pane_id.clone(),
                title: non_empty(record.pane_title, record.pane_id.as_str()),
                current_command: record.current_command,
                preview,
                agent: harness.map(|harness| AgentSnapshot {
                    harness,
                    status: AgentStatus::Unknown,
                    integration_mode: IntegrationMode::Compatibility,
                    activity_score: 0,
                }),
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

        Ok(Inventory::new(sessions))
    }
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
}

fn parse_pane_record(line: &str) -> Result<TmuxPaneRecord, TmuxError> {
    let mut fields = line.splitn(7, '\t');
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

    Ok(TmuxPaneRecord {
        session_id: SessionId::new(session_id),
        session_name: session_name.to_string(),
        window_id: WindowId::new(window_id),
        window_name: window_name.to_string(),
        pane_id: PaneId::new(pane_id),
        pane_title: pane_title.to_string(),
        current_command: (!current_command.trim().is_empty()).then(|| current_command.to_string()),
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

#[cfg(test)]
mod tests {
    use super::{TmuxAdapter, TmuxBackend, TmuxError, TmuxPaneRecord};
    use crate::app::{AppState, PaneId, SelectionTarget, SessionId, WindowId};
    use std::collections::BTreeMap;

    #[derive(Debug, Default)]
    struct FakeTmuxBackend {
        panes: Vec<TmuxPaneRecord>,
        captures: BTreeMap<PaneId, Result<String, String>>,
        list_error: Option<String>,
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
            match self.captures.get(pane_id) {
                Some(Ok(capture)) => Ok(capture.clone()),
                Some(Err(message)) => Err(TmuxError::Unavailable(message.clone())),
                None => Ok(String::new()),
            }
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
                    Some("zsh"),
                ),
                pane_record("$2", "beta", "@2", "agents", "%2", "beta-main", Some("zsh")),
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
                    Some("zsh"),
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
}
