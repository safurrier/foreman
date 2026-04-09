use crate::app::state::{
    AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory, Pane, PaneId, Session,
    SessionId, Window, WindowId,
};

#[derive(Debug, Clone)]
pub struct AgentSnapshotBuilder {
    harness: HarnessKind,
    status: AgentStatus,
    integration_mode: IntegrationMode,
    activity_score: u64,
}

impl AgentSnapshotBuilder {
    pub fn new(harness: HarnessKind) -> Self {
        Self {
            harness,
            status: AgentStatus::Unknown,
            integration_mode: IntegrationMode::Compatibility,
            activity_score: 0,
        }
    }

    pub fn status(mut self, status: AgentStatus) -> Self {
        self.status = status;
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

    pub fn build(self) -> AgentSnapshot {
        AgentSnapshot {
            harness: self.harness,
            status: self.status,
            integration_mode: self.integration_mode,
            activity_score: self.activity_score,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaneBuilder {
    id: PaneId,
    title: String,
    current_command: Option<String>,
    preview: String,
    agent: Option<AgentSnapshot>,
}

impl PaneBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            title: id.clone(),
            id: PaneId::new(id),
            current_command: None,
            preview: String::new(),
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

    pub fn preview(mut self, preview: impl Into<String>) -> Self {
        self.preview = preview.into();
        self
    }

    pub fn status(mut self, status: AgentStatus) -> Self {
        self.agent_mut().status = status;
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

    pub fn build(self) -> Pane {
        Pane {
            id: self.id,
            title: self.title,
            current_command: self.current_command,
            preview: self.preview,
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
