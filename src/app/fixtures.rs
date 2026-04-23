use crate::app::state::{
    AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory, Pane, PaneId,
    PreviewProvenance, Session, SessionId, Window, WindowId,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AgentSnapshotBuilder {
    harness: HarnessKind,
    status: AgentStatus,
    observed_status: AgentStatus,
    integration_mode: IntegrationMode,
    activity_score: u64,
    debounce_ticks: u8,
}

impl AgentSnapshotBuilder {
    pub fn new(harness: HarnessKind) -> Self {
        Self {
            harness,
            status: AgentStatus::Unknown,
            observed_status: AgentStatus::Unknown,
            integration_mode: IntegrationMode::Compatibility,
            activity_score: 0,
            debounce_ticks: 0,
        }
    }

    pub fn status(mut self, status: AgentStatus) -> Self {
        self.status = status;
        self.observed_status = status;
        self
    }

    pub fn observed_status(mut self, observed_status: AgentStatus) -> Self {
        self.observed_status = observed_status;
        self
    }

    pub fn integration_mode(mut self, integration_mode: IntegrationMode) -> Self {
        self.integration_mode = integration_mode;
        self
    }

    pub fn activity_score(mut self, activity_score: u64) -> Self {
        self.activity_score = activity_score;
        self
    }

    pub fn debounce_ticks(mut self, debounce_ticks: u8) -> Self {
        self.debounce_ticks = debounce_ticks;
        self
    }

    pub fn build(self) -> AgentSnapshot {
        AgentSnapshot {
            harness: self.harness,
            status: self.status,
            observed_status: self.observed_status,
            integration_mode: self.integration_mode,
            activity_score: self.activity_score,
            debounce_ticks: self.debounce_ticks,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaneBuilder {
    id: PaneId,
    title: String,
    current_command: Option<String>,
    working_dir: Option<PathBuf>,
    preview: String,
    preview_provenance: PreviewProvenance,
    agent: Option<AgentSnapshot>,
}

impl PaneBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            title: id.clone(),
            id: PaneId::new(id),
            current_command: None,
            working_dir: None,
            preview: String::new(),
            preview_provenance: PreviewProvenance::Captured,
            agent: None,
        }
    }

    pub fn agent(id: impl Into<String>, harness: HarnessKind) -> Self {
        Self::new(id).with_agent(AgentSnapshotBuilder::new(harness).build())
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_agent(mut self, agent: AgentSnapshot) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn current_command(mut self, current_command: impl Into<String>) -> Self {
        self.current_command = Some(current_command.into());
        self
    }

    pub fn working_dir(mut self, working_dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    pub fn preview(mut self, preview: impl Into<String>) -> Self {
        self.preview = preview.into();
        self
    }

    pub fn preview_provenance(mut self, preview_provenance: PreviewProvenance) -> Self {
        self.preview_provenance = preview_provenance;
        self
    }

    pub fn status(mut self, status: AgentStatus) -> Self {
        self.agent_mut().status = status;
        self.agent_mut().observed_status = status;
        self
    }

    pub fn observed_status(mut self, observed_status: AgentStatus) -> Self {
        self.agent_mut().observed_status = observed_status;
        self
    }

    pub fn integration_mode(mut self, integration_mode: IntegrationMode) -> Self {
        self.agent_mut().integration_mode = integration_mode;
        self
    }

    pub fn activity_score(mut self, activity_score: u64) -> Self {
        self.agent_mut().activity_score = activity_score;
        self
    }

    pub fn debounce_ticks(mut self, debounce_ticks: u8) -> Self {
        self.agent_mut().debounce_ticks = debounce_ticks;
        self
    }

    pub fn build(self) -> Pane {
        Pane {
            id: self.id,
            title: self.title,
            current_command: self.current_command,
            working_dir: self.working_dir,
            preview: self.preview,
            preview_provenance: self.preview_provenance,
            agent: self.agent,
        }
    }

    fn agent_mut(&mut self) -> &mut AgentSnapshot {
        self.agent
            .as_mut()
            .expect("pane builder must be initialized as an agent before mutating agent fields")
    }
}

#[derive(Debug, Clone)]
pub struct WindowBuilder {
    id: WindowId,
    name: String,
    panes: Vec<PaneBuilder>,
}

impl WindowBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id: WindowId::new(id),
            panes: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn pane(mut self, pane: PaneBuilder) -> Self {
        self.panes.push(pane);
        self
    }

    pub fn build(self) -> Window {
        Window {
            id: self.id,
            name: self.name,
            panes: self.panes.into_iter().map(PaneBuilder::build).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionBuilder {
    id: SessionId,
    name: String,
    windows: Vec<WindowBuilder>,
}

impl SessionBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id: SessionId::new(id),
            windows: Vec::new(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn window(mut self, window: WindowBuilder) -> Self {
        self.windows.push(window);
        self
    }

    pub fn build(self) -> Session {
        Session {
            id: self.id,
            name: self.name,
            windows: self.windows.into_iter().map(WindowBuilder::build).collect(),
        }
    }
}

pub fn inventory(sessions: impl IntoIterator<Item = SessionBuilder>) -> Inventory {
    Inventory::new(sessions.into_iter().map(SessionBuilder::build).collect())
}
