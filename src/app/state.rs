use std::cmp::Ordering;
use std::collections::BTreeSet;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

id_type!(SessionId);
id_type!(WindowId);
id_type!(PaneId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    PreviewScroll,
    Input,
    Spawn,
    Search,
    FlashNavigate,
    Rename,
    Help,
    ConfirmKill,
}

impl Mode {
    pub fn precedence(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::PreviewScroll => 1,
            Self::Input => 2,
            Self::Spawn => 3,
            Self::Search => 4,
            Self::FlashNavigate => 5,
            Self::Rename => 6,
            Self::Help => 7,
            Self::ConfirmKill => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sidebar,
    Preview,
    Input,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Self::Sidebar => Self::Preview,
            Self::Preview => Self::Input,
            Self::Input => Self::Sidebar,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Sidebar => Self::Input,
            Self::Preview => Self::Sidebar,
            Self::Input => Self::Preview,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    RecentActivity,
    AttentionFirst,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessKind {
    ClaudeCode,
    CodexCli,
    GeminiCli,
    OpenCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentStatus {
    Working,
    NeedsAttention,
    Idle,
    Error,
    #[default]
    Unknown,
}

impl AgentStatus {
    pub fn attention_rank(self) -> u8 {
        match self {
            Self::Error => 0,
            Self::NeedsAttention => 1,
            Self::Working => 2,
            Self::Idle => 3,
            Self::Unknown => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationMode {
    Native,
    Compatibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSnapshot {
    pub harness: HarnessKind,
    pub status: AgentStatus,
    pub observed_status: AgentStatus,
    pub integration_mode: IntegrationMode,
    pub activity_score: u64,
    pub debounce_ticks: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pane {
    pub id: PaneId,
    pub title: String,
    pub current_command: Option<String>,
    pub preview: String,
    pub agent: Option<AgentSnapshot>,
}

impl Pane {
    pub fn is_agent(&self) -> bool {
        self.agent.is_some()
    }

    pub fn is_visible(&self, filters: &Filters) -> bool {
        self.is_agent() || filters.show_non_agent_panes
    }

    pub fn recent_activity(&self) -> u64 {
        self.agent
            .as_ref()
            .map(|agent| agent.activity_score)
            .unwrap_or_default()
    }

    pub fn attention_rank(&self) -> u8 {
        self.agent
            .as_ref()
            .map(|agent| agent.status.attention_rank())
            .unwrap_or_else(|| AgentStatus::Unknown.attention_rank())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    pub id: WindowId,
    pub name: String,
    pub panes: Vec<Pane>,
}

impl Window {
    pub fn has_agent_panes(&self) -> bool {
        self.panes.iter().any(Pane::is_agent)
    }

    pub fn has_visible_panes(&self, filters: &Filters) -> bool {
        self.panes.iter().any(|pane| pane.is_visible(filters))
    }

    pub fn visible_panes(&self, filters: &Filters, sort_mode: SortMode) -> Vec<&Pane> {
        let mut panes: Vec<&Pane> = self
            .panes
            .iter()
            .filter(|pane| pane.is_visible(filters))
            .collect();
        panes.sort_by(|left, right| pane_cmp(left, right, sort_mode));
        panes
    }

    pub fn recent_activity(&self) -> u64 {
        self.panes
            .iter()
            .map(Pane::recent_activity)
            .max()
            .unwrap_or_default()
    }

    pub fn attention_rank(&self) -> u8 {
        self.panes
            .iter()
            .map(Pane::attention_rank)
            .min()
            .unwrap_or_else(|| AgentStatus::Unknown.attention_rank())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub windows: Vec<Window>,
}

impl Session {
    pub fn has_agent_panes(&self) -> bool {
        self.windows.iter().any(Window::has_agent_panes)
    }

    pub fn is_visible(&self, filters: &Filters) -> bool {
        self.has_agent_panes() || filters.show_non_agent_sessions
    }

    pub fn visible_windows(&self, filters: &Filters, sort_mode: SortMode) -> Vec<&Window> {
        let force_windows_visible = filters.show_non_agent_sessions && !self.has_agent_panes();
        let mut windows: Vec<&Window> = self
            .windows
            .iter()
            .filter(|window| {
                if force_windows_visible {
                    !window.panes.is_empty()
                } else {
                    window.has_visible_panes(filters)
                }
            })
            .collect();
        windows.sort_by(|left, right| window_cmp(left, right, sort_mode));
        windows
    }

    pub fn recent_activity(&self) -> u64 {
        self.windows
            .iter()
            .map(Window::recent_activity)
            .max()
            .unwrap_or_default()
    }

    pub fn attention_rank(&self) -> u8 {
        self.windows
            .iter()
            .map(Window::attention_rank)
            .min()
            .unwrap_or_else(|| AgentStatus::Unknown.attention_rank())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inventory {
    pub sessions: Vec<Session>,
}

impl Inventory {
    pub fn new(sessions: Vec<Session>) -> Self {
        Self { sessions }
    }

    pub fn contains_session(&self, session_id: &SessionId) -> bool {
        self.sessions
            .iter()
            .any(|session| &session.id == session_id)
    }

    pub fn contains_target(&self, target: &SelectionTarget) -> bool {
        match target {
            SelectionTarget::Session(session_id) => self.contains_session(session_id),
            SelectionTarget::Window(window_id) => self.parent_for_window(window_id).is_some(),
            SelectionTarget::Pane(pane_id) => self.parent_for_pane(pane_id).is_some(),
        }
    }

    pub fn session(&self, session_id: &SessionId) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| &session.id == session_id)
    }

    pub fn window(&self, window_id: &WindowId) -> Option<&Window> {
        self.sessions
            .iter()
            .flat_map(|session| session.windows.iter())
            .find(|window| &window.id == window_id)
    }

    pub fn pane(&self, pane_id: &PaneId) -> Option<&Pane> {
        self.sessions
            .iter()
            .flat_map(|session| session.windows.iter())
            .flat_map(|window| window.panes.iter())
            .find(|pane| &pane.id == pane_id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn window_count(&self) -> usize {
        self.sessions
            .iter()
            .map(|session| session.windows.len())
            .sum()
    }

    pub fn pane_count(&self) -> usize {
        self.sessions
            .iter()
            .flat_map(|session| session.windows.iter())
            .map(|window| window.panes.len())
            .sum()
    }

    pub fn visible_targets(
        &self,
        filters: &Filters,
        collapsed_sessions: &BTreeSet<SessionId>,
        sort_mode: SortMode,
    ) -> Vec<SelectionTarget> {
        let mut targets = Vec::new();

        for session in self.sorted_sessions(sort_mode) {
            if !session.is_visible(filters) {
                continue;
            }

            targets.push(SelectionTarget::Session(session.id.clone()));

            if collapsed_sessions.contains(&session.id) {
                continue;
            }

            for window in session.visible_windows(filters, sort_mode) {
                targets.push(SelectionTarget::Window(window.id.clone()));

                for pane in window.visible_panes(filters, sort_mode) {
                    targets.push(SelectionTarget::Pane(pane.id.clone()));
                }
            }
        }

        targets
    }

    pub fn reconcile_selection(
        &self,
        current: Option<&SelectionTarget>,
        filters: &Filters,
        collapsed_sessions: &BTreeSet<SessionId>,
        sort_mode: SortMode,
    ) -> Option<SelectionTarget> {
        let visible_targets = self.visible_targets(filters, collapsed_sessions, sort_mode);
        if visible_targets.is_empty() {
            return None;
        }

        if let Some(current) = current {
            if visible_targets.contains(current) {
                return Some(current.clone());
            }
        }

        if let Some(current) = current {
            if let Some(ancestor) = self.visible_ancestor(current, &visible_targets) {
                return Some(ancestor);
            }
        }

        Some(visible_targets[0].clone())
    }

    fn visible_ancestor(
        &self,
        target: &SelectionTarget,
        visible_targets: &[SelectionTarget],
    ) -> Option<SelectionTarget> {
        match target {
            SelectionTarget::Session(session_id) => {
                let session_target = SelectionTarget::Session(session_id.clone());
                visible_targets
                    .contains(&session_target)
                    .then_some(session_target)
            }
            SelectionTarget::Window(window_id) => {
                self.parent_for_window(window_id).and_then(|session| {
                    let session_target = SelectionTarget::Session(session.id.clone());
                    visible_targets
                        .contains(&session_target)
                        .then_some(session_target)
                })
            }
            SelectionTarget::Pane(pane_id) => {
                self.parent_for_pane(pane_id).and_then(|(session, window)| {
                    let window_target = SelectionTarget::Window(window.id.clone());
                    if visible_targets.contains(&window_target) {
                        return Some(window_target);
                    }

                    let session_target = SelectionTarget::Session(session.id.clone());
                    visible_targets
                        .contains(&session_target)
                        .then_some(session_target)
                })
            }
        }
    }

    fn sorted_sessions(&self, sort_mode: SortMode) -> Vec<&Session> {
        let mut sessions: Vec<&Session> = self.sessions.iter().collect();
        sessions.sort_by(|left, right| session_cmp(left, right, sort_mode));
        sessions
    }

    fn parent_for_window(&self, window_id: &WindowId) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|session| session.windows.iter().any(|window| &window.id == window_id))
    }

    fn parent_for_pane(&self, pane_id: &PaneId) -> Option<(&Session, &Window)> {
        for session in &self.sessions {
            for window in &session.windows {
                if window.panes.iter().any(|pane| &pane.id == pane_id) {
                    return Some((session, window));
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SelectionTarget {
    Session(SessionId),
    Window(WindowId),
    Pane(PaneId),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Filters {
    pub show_non_agent_sessions: bool,
    pub show_non_agent_panes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextDraft {
    pub text: String,
}

impl TextDraft {
    pub fn from_text(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn push_char(&mut self, ch: char) {
        self.text.push(ch);
    }

    pub fn insert_newline(&mut self) {
        self.text.push('\n');
    }

    pub fn backspace(&mut self) {
        self.text.pop();
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn take(&mut self) -> String {
        std::mem::take(&mut self.text)
    }

    pub fn is_blank(&self) -> bool {
        self.text.trim().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    RenameWindow {
        window_id: WindowId,
        draft: TextDraft,
    },
    SpawnWindow {
        session_id: SessionId,
        draft: TextDraft,
    },
    ConfirmKill {
        pane_id: PaneId,
    },
}

impl ModalState {
    pub fn rename_window(window_id: WindowId, draft: impl Into<String>) -> Self {
        Self::RenameWindow {
            window_id,
            draft: TextDraft::from_text(draft),
        }
    }

    pub fn spawn_window(session_id: SessionId) -> Self {
        Self::SpawnWindow {
            session_id,
            draft: TextDraft::default(),
        }
    }

    pub fn confirm_kill(pane_id: PaneId) -> Self {
        Self::ConfirmKill { pane_id }
    }

    pub fn draft(&self) -> Option<&TextDraft> {
        match self {
            Self::RenameWindow { draft, .. } | Self::SpawnWindow { draft, .. } => Some(draft),
            Self::ConfirmKill { .. } => None,
        }
    }

    pub fn draft_mut(&mut self) -> Option<&mut TextDraft> {
        match self {
            Self::RenameWindow { draft, .. } | Self::SpawnWindow { draft, .. } => Some(draft),
            Self::ConfirmKill { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AppState {
    pub inventory: Inventory,
    pub selection: Option<SelectionTarget>,
    pub focus: Focus,
    pub mode: Mode,
    pub popup_mode: bool,
    pub sort_mode: SortMode,
    pub filters: Filters,
    pub collapsed_sessions: BTreeSet<SessionId>,
    pub search_query: String,
    pub input_draft: TextDraft,
    pub modal: Option<ModalState>,
    pub startup_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventorySummary {
    pub total_sessions: usize,
    pub total_windows: usize,
    pub total_panes: usize,
    pub visible_sessions: usize,
    pub visible_windows: usize,
    pub visible_panes: usize,
    pub startup_error: Option<String>,
}

impl AppState {
    pub fn with_inventory(inventory: Inventory) -> Self {
        let mut state = Self {
            inventory,
            ..Self::default()
        };
        state.reconcile_selection();
        state
    }

    pub fn visible_targets(&self) -> Vec<SelectionTarget> {
        self.inventory
            .visible_targets(&self.filters, &self.collapsed_sessions, self.sort_mode)
    }

    pub fn reconcile_selection(&mut self) {
        self.selection = self.inventory.reconcile_selection(
            self.selection.as_ref(),
            &self.filters,
            &self.collapsed_sessions,
            self.sort_mode,
        );
    }

    pub fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Normal => "NORMAL",
            Mode::PreviewScroll => "PREVIEW",
            Mode::Input => "INPUT",
            Mode::Spawn => "SPAWN",
            Mode::Search => "SEARCH",
            Mode::FlashNavigate => "FLASH",
            Mode::Rename => "RENAME",
            Mode::Help => "HELP",
            Mode::ConfirmKill => "CONFIRM KILL",
        }
    }

    pub fn focus_label(&self) -> &'static str {
        match self.focus {
            Focus::Sidebar => "SIDEBAR",
            Focus::Preview => "PREVIEW",
            Focus::Input => "INPUT",
        }
    }

    pub fn inventory_summary(&self) -> InventorySummary {
        let visible_targets = self.visible_targets();
        InventorySummary {
            total_sessions: self.inventory.session_count(),
            total_windows: self.inventory.window_count(),
            total_panes: self.inventory.pane_count(),
            visible_sessions: visible_targets
                .iter()
                .filter(|target| matches!(target, SelectionTarget::Session(_)))
                .count(),
            visible_windows: visible_targets
                .iter()
                .filter(|target| matches!(target, SelectionTarget::Window(_)))
                .count(),
            visible_panes: visible_targets
                .iter()
                .filter(|target| matches!(target, SelectionTarget::Pane(_)))
                .count(),
            startup_error: self.startup_error.clone(),
        }
    }

    pub fn selected_pane_id(&self) -> Option<PaneId> {
        match self.selection.as_ref()? {
            SelectionTarget::Pane(pane_id) => Some(pane_id.clone()),
            SelectionTarget::Session(_) | SelectionTarget::Window(_) => None,
        }
    }

    pub fn selected_window_id(&self) -> Option<WindowId> {
        match self.selection.as_ref()? {
            SelectionTarget::Window(window_id) => Some(window_id.clone()),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .parent_for_pane(pane_id)
                .map(|(_, window)| window.id.clone()),
            SelectionTarget::Session(_) => None,
        }
    }

    pub fn selected_window_name(&self) -> Option<String> {
        let window_id = self.selected_window_id()?;
        self.inventory
            .window(&window_id)
            .map(|window| window.name.clone())
    }

    pub fn selected_session_id(&self) -> Option<SessionId> {
        match self.selection.as_ref()? {
            SelectionTarget::Session(session_id) => Some(session_id.clone()),
            SelectionTarget::Window(window_id) => self
                .inventory
                .parent_for_window(window_id)
                .map(|session| session.id.clone()),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .parent_for_pane(pane_id)
                .map(|(session, _)| session.id.clone()),
        }
    }
}

fn session_cmp(left: &Session, right: &Session, sort_mode: SortMode) -> Ordering {
    match sort_mode {
        SortMode::RecentActivity => right
            .recent_activity()
            .cmp(&left.recent_activity())
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
        SortMode::AttentionFirst => left
            .attention_rank()
            .cmp(&right.attention_rank())
            .then_with(|| right.recent_activity().cmp(&left.recent_activity()))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
    }
}

fn window_cmp(left: &Window, right: &Window, sort_mode: SortMode) -> Ordering {
    match sort_mode {
        SortMode::RecentActivity => right
            .recent_activity()
            .cmp(&left.recent_activity())
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
        SortMode::AttentionFirst => left
            .attention_rank()
            .cmp(&right.attention_rank())
            .then_with(|| right.recent_activity().cmp(&left.recent_activity()))
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
    }
}

fn pane_cmp(left: &Pane, right: &Pane, sort_mode: SortMode) -> Ordering {
    match sort_mode {
        SortMode::RecentActivity => right
            .recent_activity()
            .cmp(&left.recent_activity())
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
        SortMode::AttentionFirst => left
            .attention_rank()
            .cmp(&right.attention_rank())
            .then_with(|| right.recent_activity().cmp(&left.recent_activity()))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
    }
}
