use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::services::pull_requests::{PullRequestData, PullRequestLookup};
use crate::services::system_stats::SystemStatsSnapshot;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotificationKind {
    Completion,
    NeedsAttention,
}

impl NotificationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Completion => "completion",
            Self::NeedsAttention => "needs_attention",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationProfile {
    #[default]
    All,
    CompletionOnly,
    AttentionOnly,
}

impl NotificationProfile {
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::CompletionOnly,
            Self::CompletionOnly => Self::AttentionOnly,
            Self::AttentionOnly => Self::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "ALL",
            Self::CompletionOnly => "COMPLETE",
            Self::AttentionOnly => "ATTENTION",
        }
    }

    pub fn allows(self, kind: NotificationKind) -> bool {
        match self {
            Self::All => true,
            Self::CompletionOnly => kind == NotificationKind::Completion,
            Self::AttentionOnly => kind == NotificationKind::NeedsAttention,
        }
    }
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
    pub working_dir: Option<PathBuf>,
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
pub struct SearchState {
    pub draft: TextDraft,
    pub restore_selection: Option<SelectionTarget>,
}

impl SearchState {
    pub fn new(restore_selection: Option<SelectionTarget>) -> Self {
        Self {
            draft: TextDraft::default(),
            restore_selection,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashNavigateKind {
    Jump,
    JumpAndFocus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashState {
    pub draft: TextDraft,
    pub restore_selection: Option<SelectionTarget>,
    pub kind: FlashNavigateKind,
}

impl FlashState {
    pub fn new(restore_selection: Option<SelectionTarget>, kind: FlashNavigateKind) -> Self {
        Self {
            draft: TextDraft::default(),
            restore_selection,
            kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlashTarget {
    pub label: String,
    pub target: SelectionTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NotificationCooldownKey {
    pub pane_id: PaneId,
    pub kind: NotificationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationState {
    pub muted: bool,
    pub profile: NotificationProfile,
    pub refresh_tick: u64,
    pub cooldown_ticks: u64,
    pub cooldowns: BTreeMap<NotificationCooldownKey, u64>,
    pub last_status: Option<String>,
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            muted: false,
            profile: NotificationProfile::default(),
            refresh_tick: 0,
            cooldown_ticks: crate::config::DEFAULT_NOTIFICATION_COOLDOWN_TICKS,
            cooldowns: BTreeMap::new(),
            last_status: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorAlertLevel {
    Info,
    Warn,
    Error,
}

impl OperatorAlertLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatorAlertSource {
    Tmux,
    PullRequests,
    Notifications,
    Browser,
    Clipboard,
}

impl OperatorAlertSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tmux => "tmux",
            Self::PullRequests => "pull_requests",
            Self::Notifications => "notifications",
            Self::Browser => "browser",
            Self::Clipboard => "clipboard",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAlert {
    pub source: OperatorAlertSource,
    pub level: OperatorAlertLevel,
    pub message: String,
}

impl OperatorAlert {
    pub fn new(
        source: OperatorAlertSource,
        level: OperatorAlertLevel,
        message: impl Into<String>,
    ) -> Self {
        Self {
            source,
            level,
            message: message.into(),
        }
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
    pub search: Option<SearchState>,
    pub flash: Option<FlashState>,
    pub pull_request_cache: BTreeMap<PathBuf, PullRequestLookup>,
    pub pull_request_detail_workspace: Option<PathBuf>,
    pub pull_request_detail_manual: bool,
    pub pull_request_auto_open_dismissed: BTreeSet<PathBuf>,
    pub notifications: NotificationState,
    pub input_draft: TextDraft,
    pub modal: Option<ModalState>,
    pub system_stats: SystemStatsSnapshot,
    pub operator_alert: Option<OperatorAlert>,
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
        let targets = self.base_visible_targets();
        self.filter_search_targets(targets)
    }

    pub fn base_visible_targets(&self) -> Vec<SelectionTarget> {
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

    pub fn search_query(&self) -> Option<&str> {
        self.search
            .as_ref()
            .map(|search| search.draft.text.as_str())
    }

    pub fn target_label(&self, target: &SelectionTarget) -> String {
        match target {
            SelectionTarget::Session(session_id) => self
                .inventory
                .session(session_id)
                .map(|session| format!("Session  {}", session.name))
                .unwrap_or_else(|| format!("Session  {}", session_id.as_str())),
            SelectionTarget::Window(window_id) => self
                .inventory
                .window(window_id)
                .map(|window| format!("Window   {}", window.name))
                .unwrap_or_else(|| format!("Window   {}", window_id.as_str())),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .pane(pane_id)
                .map(|pane| {
                    let status = pane
                        .agent
                        .as_ref()
                        .map(|agent| match agent.status {
                            AgentStatus::Working => "WORKING",
                            AgentStatus::NeedsAttention => "ATTN",
                            AgentStatus::Idle => "IDLE",
                            AgentStatus::Error => "ERROR",
                            AgentStatus::Unknown => "UNKNOWN",
                        })
                        .unwrap_or("NON-AGENT");
                    format!("Pane     {} [{}]", pane.title, status)
                })
                .unwrap_or_else(|| format!("Pane     {}", pane_id.as_str())),
        }
    }

    pub fn flash_targets(&self) -> Vec<FlashTarget> {
        let targets = self.base_visible_targets();
        let width = flash_label_width(targets.len());
        targets
            .into_iter()
            .enumerate()
            .map(|(index, target)| FlashTarget {
                label: flash_label_for_index(index, width),
                target,
            })
            .collect()
    }

    pub fn flash_label_for_target(&self, target: &SelectionTarget) -> Option<String> {
        self.flash_targets()
            .into_iter()
            .find(|candidate| &candidate.target == target)
            .map(|candidate| candidate.label)
    }

    pub fn matching_flash_targets(&self) -> Vec<FlashTarget> {
        let Some(flash) = self.flash.as_ref() else {
            return Vec::new();
        };

        let input = flash.draft.text.to_ascii_lowercase();
        if input.is_empty() {
            return self.flash_targets();
        }

        self.flash_targets()
            .into_iter()
            .filter(|candidate| candidate.label.starts_with(&input))
            .collect()
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

    pub fn selected_workspace_path(&self) -> Option<PathBuf> {
        let target = self.selection.as_ref()?;
        let mut paths = self.workspace_paths_for_target(target).into_iter();
        let first = paths.next()?;
        if paths.next().is_some() {
            return None;
        }
        Some(first)
    }

    pub fn selected_pull_request_lookup(&self) -> Option<&PullRequestLookup> {
        let workspace_path = self.selected_workspace_path()?;
        self.pull_request_cache.get(&workspace_path)
    }

    pub fn selected_pull_request(&self) -> Option<&PullRequestData> {
        match self.selected_pull_request_lookup()? {
            PullRequestLookup::Available(pull_request) => Some(pull_request),
            PullRequestLookup::Unknown
            | PullRequestLookup::Missing
            | PullRequestLookup::Unavailable { .. } => None,
        }
    }

    pub fn is_pull_request_detail_open(&self) -> bool {
        self.selected_workspace_path()
            .as_ref()
            .zip(self.pull_request_detail_workspace.as_ref())
            .is_some_and(|(selected_workspace, open_workspace)| {
                selected_workspace == open_workspace
            })
            && self.selected_pull_request().is_some()
    }

    pub fn pull_request_compact_label(&self) -> Option<String> {
        match self.selected_pull_request_lookup()? {
            PullRequestLookup::Unknown => Some("pr=CHECKING".to_string()),
            PullRequestLookup::Missing => Some("pr=NONE".to_string()),
            PullRequestLookup::Unavailable { .. } => Some("pr=UNAVAILABLE".to_string()),
            PullRequestLookup::Available(pull_request) => Some(format!(
                "pr=#{} {}",
                pull_request.number,
                pull_request.status.label()
            )),
        }
    }

    pub fn notifications_label(&self) -> String {
        if self.notifications.muted {
            "notify=MUTED".to_string()
        } else {
            format!("notify={}", self.notifications.profile.label())
        }
    }

    pub fn system_stats_label(&self) -> String {
        self.system_stats.label()
    }

    pub fn operator_alert_label(&self) -> Option<String> {
        self.operator_alert
            .as_ref()
            .map(|alert| format!("alert={}", alert.level.label()))
    }

    fn filter_search_targets(&self, targets: Vec<SelectionTarget>) -> Vec<SelectionTarget> {
        let Some(query) = self.search_query().map(str::trim) else {
            return targets;
        };
        if query.is_empty() {
            return targets;
        }

        let needle = query.to_ascii_lowercase();
        targets
            .into_iter()
            .filter(|target| {
                self.target_label(target)
                    .to_ascii_lowercase()
                    .contains(&needle)
            })
            .collect()
    }

    fn workspace_paths_for_target(&self, target: &SelectionTarget) -> BTreeSet<PathBuf> {
        match target {
            SelectionTarget::Session(session_id) => self
                .inventory
                .session(session_id)
                .into_iter()
                .flat_map(|session| session.windows.iter())
                .flat_map(|window| window.panes.iter())
                .filter_map(|pane| pane.working_dir.clone())
                .collect(),
            SelectionTarget::Window(window_id) => self
                .inventory
                .window(window_id)
                .into_iter()
                .flat_map(|window| window.panes.iter())
                .filter_map(|pane| pane.working_dir.clone())
                .collect(),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .pane(pane_id)
                .and_then(|pane| pane.working_dir.clone())
                .into_iter()
                .collect(),
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

fn flash_label_width(count: usize) -> usize {
    let mut width = 1;
    let mut capacity = 26usize;
    while count > capacity {
        width += 1;
        capacity *= 26;
    }
    width
}

fn flash_label_for_index(mut index: usize, width: usize) -> String {
    let mut chars = vec!['a'; width];
    for position in (0..width).rev() {
        chars[position] = ((index % 26) as u8 + b'a') as char;
        index /= 26;
    }
    chars.into_iter().collect()
}
