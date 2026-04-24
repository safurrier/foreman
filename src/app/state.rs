use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::doctor::DoctorFinding;
use crate::services::pull_requests::{PullRequestData, PullRequestLookup};
use crate::services::system_stats::SystemStatsSnapshot;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
    Stable,
    AttentionFirst,
}

impl SortMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::AttentionFirst => "attention->recent",
        }
    }

    pub fn help_label(self) -> &'static str {
        match self {
            Self::Stable => "stable by name",
            Self::AttentionFirst => "attention first, then recent",
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HarnessKind {
    ClaudeCode,
    CodexCli,
    Pi,
    GeminiCli,
    OpenCode,
}

impl HarnessKind {
    pub const ALL: [Self; 5] = [
        Self::ClaudeCode,
        Self::CodexCli,
        Self::Pi,
        Self::GeminiCli,
        Self::OpenCode,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::CodexCli => "Codex CLI",
            Self::Pi => "Pi",
            Self::GeminiCli => "Gemini CLI",
            Self::OpenCode => "OpenCode",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "CLD",
            Self::CodexCli => "CDX",
            Self::Pi => "PI",
            Self::GeminiCli => "GEM",
            Self::OpenCode => "OCD",
        }
    }

    pub fn filter_label(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude",
            Self::CodexCli => "codex",
            Self::Pi => "pi",
            Self::GeminiCli => "gemini",
            Self::OpenCode => "opencode",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    Working,
    NeedsAttention,
    Idle,
    Error,
    #[default]
    Unknown,
}

impl AgentStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Working => "WORKING",
            Self::NeedsAttention => "ATTENTION",
            Self::Idle => "IDLE",
            Self::Error => "ERROR",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Working => "WORK",
            Self::NeedsAttention => "ATTN",
            Self::Idle => "IDLE",
            Self::Error => "ERR",
            Self::Unknown => "UNK",
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntegrationMode {
    Native,
    Compatibility,
}

impl IntegrationMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Compatibility => "compat",
        }
    }

    pub fn source_label(self) -> &'static str {
        match self {
            Self::Native => "native hook",
            Self::Compatibility => "compatibility heuristic",
        }
    }

    pub fn confidence_label(self) -> &'static str {
        match self {
            Self::Native => "high confidence",
            Self::Compatibility => "lower confidence",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSnapshot {
    pub harness: HarnessKind,
    pub status: AgentStatus,
    pub observed_status: AgentStatus,
    pub integration_mode: IntegrationMode,
    pub activity_score: u64,
    pub debounce_ticks: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pane {
    pub id: PaneId,
    pub title: String,
    pub current_command: Option<String>,
    pub working_dir: Option<PathBuf>,
    pub preview: String,
    pub preview_provenance: PreviewProvenance,
    pub agent: Option<AgentSnapshot>,
}

impl Pane {
    pub fn is_agent(&self) -> bool {
        self.agent.is_some()
    }

    pub fn harness_kind(&self) -> Option<HarnessKind> {
        self.agent.as_ref().map(|agent| agent.harness)
    }

    pub fn is_visible(&self, filters: &Filters) -> bool {
        match self.harness_kind() {
            Some(harness) => filters.matches_harness(harness),
            None => filters.show_non_agent_panes && filters.harness.is_none(),
        }
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

    pub fn workspace_name(&self) -> Option<String> {
        self.working_dir
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned())
            .filter(|name| !name.trim().is_empty())
    }

    pub fn navigation_title(&self) -> String {
        self.workspace_name()
            .or_else(|| {
                (!self.title.trim().is_empty() && self.title != self.id.as_str())
                    .then(|| self.title.clone())
            })
            .or_else(|| self.current_command.clone())
            .unwrap_or_else(|| self.id.as_str().to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreviewProvenance {
    Captured,
    PendingCapture,
    ReusedCached,
    CaptureFailed,
}

impl PreviewProvenance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::PendingCapture => "pending capture",
            Self::ReusedCached => "reused cached preview",
            Self::CaptureFailed => "capture failed",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Self::Captured => "Preview source: captured from tmux on the latest refresh.",
            Self::PendingCapture => {
                "Preview source: tmux preview capture is queued after startup or viewport-priority refresh."
            }
            Self::ReusedCached => {
                "Preview source: reused cached preview while tmux refresh prioritized other panes."
            }
            Self::CaptureFailed => {
                "Preview source: tmux capture failed on the latest refresh; pane metadata is still live."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

    pub fn navigation_title(&self) -> String {
        if !is_generic_window_name(&self.name) {
            return self.name.clone();
        }

        let workspace_names = self
            .panes
            .iter()
            .filter_map(Pane::workspace_name)
            .collect::<BTreeSet<_>>();
        if workspace_names.len() == 1 {
            return workspace_names
                .into_iter()
                .next()
                .expect("single workspace should exist");
        }

        if self.panes.len() == 1 {
            return self.panes[0].navigation_title();
        }

        self.name.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub windows: Vec<Window>,
}

impl Session {
    pub fn has_agent_panes(&self) -> bool {
        self.windows.iter().any(Window::has_agent_panes)
    }

    pub fn has_matching_agent_panes(&self, filters: &Filters) -> bool {
        self.windows
            .iter()
            .flat_map(|window| window.panes.iter())
            .any(|pane| {
                pane.harness_kind()
                    .is_some_and(|harness| filters.matches_harness(harness))
            })
    }

    pub fn is_visible(&self, filters: &Filters) -> bool {
        self.has_matching_agent_panes(filters)
            || (filters.show_non_agent_sessions
                && filters.harness.is_none()
                && !self.has_agent_panes()
                && self.windows.iter().any(|window| !window.panes.is_empty()))
    }

    pub fn visible_windows(&self, filters: &Filters, sort_mode: SortMode) -> Vec<&Window> {
        let force_windows_visible =
            filters.show_non_agent_sessions && filters.harness.is_none() && !self.has_agent_panes();
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
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

    pub fn actionable_pane_for_target<'a>(
        &'a self,
        target: &SelectionTarget,
        filters: &Filters,
        sort_mode: SortMode,
    ) -> Option<&'a Pane> {
        match target {
            SelectionTarget::Pane(pane_id) => {
                self.pane(pane_id).filter(|pane| pane.is_visible(filters))
            }
            SelectionTarget::Window(window_id) => self
                .window(window_id)
                .and_then(|window| window.visible_panes(filters, sort_mode).into_iter().next()),
            SelectionTarget::Session(session_id) => self.session(session_id).and_then(|session| {
                session
                    .visible_windows(filters, sort_mode)
                    .into_iter()
                    .find_map(|window| window.visible_panes(filters, sort_mode).into_iter().next())
            }),
        }
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

    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.sessions
            .iter()
            .flat_map(|session| session.windows.iter())
            .flat_map(|window| window.panes.iter())
            .map(|pane| pane.id.clone())
            .collect()
    }

    pub fn available_harnesses(&self) -> Vec<HarnessKind> {
        HarnessKind::ALL
            .into_iter()
            .filter(|candidate| {
                self.sessions
                    .iter()
                    .flat_map(|session| session.windows.iter())
                    .flat_map(|window| window.panes.iter())
                    .any(|pane| {
                        pane.harness_kind()
                            .is_some_and(|harness| harness == *candidate)
                    })
            })
            .collect()
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
pub struct SidebarHarnessSummary {
    pub harnesses: Vec<HarnessKind>,
    pub saw_shell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarRowKind {
    Session {
        name: String,
        collapsed: bool,
        rank: u8,
        visible_windows: usize,
        visible_panes: usize,
        harnesses: SidebarHarnessSummary,
    },
    Window {
        name: String,
        rank: u8,
        visible_panes: usize,
        harnesses: SidebarHarnessSummary,
    },
    Pane {
        navigation_title: String,
        status: Option<AgentStatus>,
        harness: Option<HarnessKind>,
        is_agent: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTargetEntry {
    pub target: SelectionTarget,
    pub actionable_pane_id: Option<PaneId>,
    pub actionable_workspace_path: Option<PathBuf>,
    pub aggregate_workspace_path: Option<PathBuf>,
    pub sidebar: SidebarRowKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VisibleStateCache {
    pub base_targets: Vec<SelectionTarget>,
    pub flash_targets: Vec<FlashTarget>,
    pub entries: Vec<VisibleTargetEntry>,
    pub visible_sessions: usize,
    pub visible_windows: usize,
    pub visible_panes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Filters {
    pub show_non_agent_sessions: bool,
    pub show_non_agent_panes: bool,
    pub harness: Option<HarnessKind>,
}

impl Filters {
    pub fn matches_harness(&self, harness: HarnessKind) -> bool {
        self.harness.is_none_or(|selected| selected == harness)
    }

    pub fn cycle_harness(&mut self, available_harnesses: &[HarnessKind]) {
        if available_harnesses.is_empty() {
            self.harness = None;
            return;
        }

        self.harness = match self.harness {
            None => Some(available_harnesses[0]),
            Some(current) => match available_harnesses
                .iter()
                .position(|candidate| *candidate == current)
            {
                Some(index) => available_harnesses.get(index + 1).copied(),
                None => Some(available_harnesses[0]),
            },
        };
    }

    pub fn harness_label(&self) -> &'static str {
        self.harness.map(HarnessKind::filter_label).unwrap_or("all")
    }

    pub fn scope_label(&self) -> &'static str {
        match (self.show_non_agent_sessions, self.show_non_agent_panes) {
            (false, false) => "agents-only",
            (true, false) => "all-sessions",
            (false, true) => "all-panes",
            (true, true) => "all-targets",
        }
    }
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
    Diagnostics,
}

impl OperatorAlertSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tmux => "tmux",
            Self::PullRequests => "pull_requests",
            Self::Notifications => "notifications",
            Self::Browser => "browser",
            Self::Clipboard => "clipboard",
            Self::Diagnostics => "diagnostics",
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

    pub fn spawn_window_with_command(session_id: SessionId, command: impl Into<String>) -> Self {
        Self::SpawnWindow {
            session_id,
            draft: TextDraft::from_text(command),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub inventory: Inventory,
    pub selection: Option<SelectionTarget>,
    pub focus: Focus,
    pub mode: Mode,
    pub help_scroll: u16,
    pub preview_scroll: u16,
    pub sidebar_scroll: usize,
    pub sidebar_viewport_rows: usize,
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
    pub startup_loading: bool,
    pub startup_cache_age_ms: Option<u64>,
    pub startup_error: Option<String>,
    pub runtime_diagnostics: Vec<DoctorFinding>,
    pub visible_state: VisibleStateCache,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventorySummary {
    pub total_sessions: usize,
    pub total_windows: usize,
    pub total_panes: usize,
    pub visible_sessions: usize,
    pub visible_windows: usize,
    pub visible_panes: usize,
    pub startup_loading: bool,
    pub startup_cache_age_ms: Option<u64>,
    pub startup_error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            inventory: Inventory::default(),
            selection: None,
            focus: Focus::default(),
            mode: Mode::default(),
            help_scroll: 0,
            preview_scroll: 0,
            sidebar_scroll: 0,
            sidebar_viewport_rows: 0,
            popup_mode: false,
            sort_mode: SortMode::default(),
            filters: Filters::default(),
            collapsed_sessions: BTreeSet::new(),
            search: None,
            flash: None,
            pull_request_cache: BTreeMap::new(),
            pull_request_detail_workspace: None,
            pull_request_detail_manual: false,
            pull_request_auto_open_dismissed: BTreeSet::new(),
            notifications: NotificationState::default(),
            input_draft: TextDraft::default(),
            modal: None,
            system_stats: SystemStatsSnapshot::default(),
            operator_alert: None,
            startup_loading: false,
            startup_cache_age_ms: None,
            startup_error: None,
            runtime_diagnostics: Vec::new(),
            visible_state: VisibleStateCache::default(),
        };
        state.rebuild_visible_state();
        state
    }
}

impl AppState {
    pub fn with_inventory(inventory: Inventory) -> Self {
        let mut state = Self {
            inventory,
            ..Self::default()
        };
        state.rebuild_visible_state();
        state.reconcile_selection();
        state
    }

    pub fn visible_targets(&self) -> Vec<SelectionTarget> {
        self.visible_state
            .entries
            .iter()
            .map(|entry| entry.target.clone())
            .collect()
    }

    pub fn base_visible_targets(&self) -> Vec<SelectionTarget> {
        self.visible_state.base_targets.clone()
    }

    pub fn visible_target_entries(&self) -> &[VisibleTargetEntry] {
        &self.visible_state.entries
    }

    pub fn selected_visible_entry(&self) -> Option<&VisibleTargetEntry> {
        let target = self.selection.as_ref()?;
        self.visible_state
            .entries
            .iter()
            .find(|entry| &entry.target == target)
    }

    pub fn selected_visible_index(&self) -> Option<usize> {
        let target = self.selection.as_ref()?;
        self.visible_state
            .entries
            .iter()
            .position(|entry| &entry.target == target)
    }

    pub fn base_visible_targets_slice(&self) -> &[SelectionTarget] {
        &self.visible_state.base_targets
    }

    pub fn visible_target_count(&self) -> usize {
        self.visible_state.entries.len()
    }

    pub fn sidebar_scroll(&self) -> usize {
        self.sidebar_scroll
    }

    pub fn set_sidebar_viewport_rows(&mut self, rows: usize) {
        self.sidebar_viewport_rows = rows;
        self.reconcile_sidebar_scroll();
    }

    pub fn rebuild_visible_state(&mut self) {
        let base_entries = self.build_base_visible_entries();
        let base_targets = base_entries
            .iter()
            .map(|entry| entry.target.clone())
            .collect::<Vec<_>>();
        let flash_targets = build_flash_targets(&base_targets);
        let entries = self.filter_search_entries(base_entries);
        let visible_sessions = entries
            .iter()
            .filter(|entry| matches!(entry.target, SelectionTarget::Session(_)))
            .count();
        let visible_windows = entries
            .iter()
            .filter(|entry| matches!(entry.target, SelectionTarget::Window(_)))
            .count();
        let visible_panes = entries
            .iter()
            .filter(|entry| matches!(entry.target, SelectionTarget::Pane(_)))
            .count();

        self.visible_state = VisibleStateCache {
            base_targets,
            flash_targets,
            entries,
            visible_sessions,
            visible_windows,
            visible_panes,
        };
        self.reconcile_sidebar_scroll();
    }

    pub fn reconcile_selection(&mut self) {
        self.selection = self.inventory.reconcile_selection(
            self.selection.as_ref(),
            &self.filters,
            &self.collapsed_sessions,
            self.sort_mode,
        );
        self.reconcile_sidebar_scroll();
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

    pub fn focus_context_label(&self) -> &'static str {
        match self.focus {
            Focus::Sidebar => "Sidebar",
            Focus::Preview => "Details",
            Focus::Input => "Compose",
        }
    }

    pub fn inventory_summary(&self) -> InventorySummary {
        InventorySummary {
            total_sessions: self.inventory.session_count(),
            total_windows: self.inventory.window_count(),
            total_panes: self.inventory.pane_count(),
            visible_sessions: self.visible_state.visible_sessions,
            visible_windows: self.visible_state.visible_windows,
            visible_panes: self.visible_state.visible_panes,
            startup_loading: self.startup_loading,
            startup_cache_age_ms: self.startup_cache_age_ms,
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
                .map(|session| format!("session {}", session.name))
                .unwrap_or_else(|| format!("session {}", session_id.as_str())),
            SelectionTarget::Window(window_id) => self
                .inventory
                .window(window_id)
                .map(|window| format!("window {}", window.name))
                .unwrap_or_else(|| format!("window {}", window_id.as_str())),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .pane(pane_id)
                .map(|pane| {
                    let harness = pane
                        .agent
                        .as_ref()
                        .map(|agent| agent.harness.label())
                        .unwrap_or("Shell");
                    let status = pane
                        .agent
                        .as_ref()
                        .map(|agent| agent.status.label())
                        .unwrap_or("NON-AGENT");
                    format!(
                        "pane {} {} {} {}",
                        pane.navigation_title(),
                        pane.title,
                        harness,
                        status
                    )
                })
                .unwrap_or_else(|| format!("pane {}", pane_id.as_str())),
        }
    }

    pub fn flash_targets(&self) -> Vec<FlashTarget> {
        self.visible_state.flash_targets.clone()
    }

    pub fn flash_label_for_target(&self, target: &SelectionTarget) -> Option<String> {
        self.visible_state
            .flash_targets
            .iter()
            .find(|candidate| &candidate.target == target)
            .cloned()
            .map(|candidate| candidate.label)
    }

    pub fn matching_flash_targets(&self) -> Vec<FlashTarget> {
        let Some(flash) = self.flash.as_ref() else {
            return Vec::new();
        };

        let input = flash.draft.text.to_ascii_lowercase();
        if input.is_empty() {
            return self.visible_state.flash_targets.clone();
        }

        self.visible_state
            .flash_targets
            .iter()
            .filter(|candidate| candidate.label.starts_with(&input))
            .cloned()
            .collect()
    }

    pub fn selected_pane_id(&self) -> Option<PaneId> {
        match self.selection.as_ref()? {
            SelectionTarget::Pane(pane_id) => Some(pane_id.clone()),
            SelectionTarget::Session(_) | SelectionTarget::Window(_) => None,
        }
    }

    pub fn selected_actionable_pane(&self) -> Option<&Pane> {
        let target = self.selection.as_ref()?;
        if let Some(entry) = self.selected_visible_entry() {
            return entry
                .actionable_pane_id
                .as_ref()
                .and_then(|pane_id| self.inventory.pane(pane_id));
        }

        self.inventory
            .actionable_pane_for_target(target, &self.filters, self.sort_mode)
    }

    pub fn selected_actionable_source_summary(&self) -> Option<String> {
        let pane = self.selected_actionable_pane()?;
        Some(match pane.agent.as_ref() {
            Some(agent) => format!(
                "{} ({})",
                agent.integration_mode.source_label(),
                agent.integration_mode.confidence_label()
            ),
            None => "plain shell pane".to_string(),
        })
    }

    pub fn reconcile_sidebar_scroll(&mut self) {
        let total_rows = self.visible_state.entries.len();
        if total_rows == 0 {
            self.sidebar_scroll = 0;
            return;
        }

        let viewport_rows = self.sidebar_viewport_rows.max(1);
        let max_scroll = total_rows.saturating_sub(viewport_rows);
        self.sidebar_scroll = self.sidebar_scroll.min(max_scroll);

        let Some(selected_index) = self.selected_visible_index() else {
            return;
        };

        if selected_index < self.sidebar_scroll {
            self.sidebar_scroll = selected_index;
        } else if selected_index >= self.sidebar_scroll + viewport_rows {
            self.sidebar_scroll = selected_index + 1 - viewport_rows;
        }

        self.sidebar_scroll = self.sidebar_scroll.min(max_scroll);
    }

    pub fn selected_actionable_pane_id(&self) -> Option<PaneId> {
        self.selected_actionable_pane().map(|pane| pane.id.clone())
    }

    pub fn selected_actionable_workspace_path(&self) -> Option<PathBuf> {
        let target = self.selection.as_ref()?;
        if let Some(entry) = self.selected_visible_entry() {
            return entry.actionable_workspace_path.clone();
        }

        match target {
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .pane(pane_id)
                .and_then(|pane| pane.working_dir.clone()),
            SelectionTarget::Session(_) | SelectionTarget::Window(_) => self
                .inventory
                .actionable_pane_for_target(target, &self.filters, self.sort_mode)
                .and_then(|pane| pane.working_dir.clone()),
        }
    }

    pub fn viewport_actionable_pane_ids(&self) -> BTreeSet<PaneId> {
        let total_rows = self.visible_state.entries.len();
        if total_rows == 0 {
            return BTreeSet::new();
        }

        let start = self.sidebar_scroll.min(total_rows);
        let end = (start + self.sidebar_viewport_rows.max(1)).min(total_rows);
        self.visible_state.entries[start..end]
            .iter()
            .filter_map(|entry| entry.actionable_pane_id.clone())
            .collect()
    }

    pub fn refresh_priority_pane_ids(&self) -> BTreeSet<PaneId> {
        let mut pane_ids = self.viewport_actionable_pane_ids();
        if let Some(pane_id) = self.selected_actionable_pane_id() {
            pane_ids.insert(pane_id);
        }

        for pane in self
            .inventory
            .sessions
            .iter()
            .flat_map(|session| session.windows.iter())
            .flat_map(|window| window.panes.iter())
        {
            if matches!(
                pane.agent.as_ref().map(|agent| agent.status),
                Some(AgentStatus::NeedsAttention | AgentStatus::Error)
            ) {
                pane_ids.insert(pane.id.clone());
            }
        }

        pane_ids
    }

    pub fn selected_runtime_diagnostics(&self) -> Vec<&DoctorFinding> {
        let provider = self
            .selected_actionable_pane()
            .and_then(|pane| pane.agent.as_ref().map(|agent| agent.harness));
        let pane_id = self
            .selected_actionable_pane()
            .map(|pane| pane.id.as_str().to_string());
        let workspace_path = self.selected_actionable_workspace_path();
        self.runtime_diagnostics
            .iter()
            .filter(|finding| {
                finding.matches_context(provider, pane_id.as_deref(), workspace_path.as_deref())
            })
            .collect()
    }

    pub fn diagnostics_summary_label(&self) -> Option<String> {
        let diagnostics = self.selected_runtime_diagnostics();
        let finding = diagnostics
            .iter()
            .find(|finding| finding.severity == crate::doctor::DoctorSeverity::Error)
            .or_else(|| diagnostics.first())
            .copied()?;
        Some(finding.summary.clone())
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
        if let Some(entry) = self.selected_visible_entry() {
            return entry.actionable_workspace_path.clone();
        }
        match target {
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .pane(pane_id)
                .and_then(|pane| pane.working_dir.clone()),
            SelectionTarget::Session(_) | SelectionTarget::Window(_) => self
                .inventory
                .actionable_pane_for_target(target, &self.filters, self.sort_mode)
                .and_then(|pane| pane.working_dir.clone()),
        }
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

    pub fn sort_label(&self) -> &'static str {
        self.sort_mode.label()
    }

    pub fn harness_filter_label(&self) -> &'static str {
        self.filters.harness_label()
    }

    pub fn filter_label(&self) -> String {
        let scope = self.filters.scope_label();
        match self.filters.harness {
            Some(harness) => format!("{} {}", harness.filter_label(), scope),
            None => scope.to_string(),
        }
    }

    pub fn selected_pull_request_panel_label(&self) -> &'static str {
        if self.is_pull_request_detail_open() {
            "open"
        } else if self.selected_pull_request().is_some() {
            "hidden"
        } else {
            "none"
        }
    }

    pub fn selection_breadcrumb(&self) -> Option<String> {
        match self.selection.as_ref()? {
            SelectionTarget::Session(session_id) => self
                .inventory
                .session(session_id)
                .map(|session| session.name.clone()),
            SelectionTarget::Window(window_id) => self
                .inventory
                .parent_for_window(window_id)
                .zip(self.inventory.window(window_id))
                .map(|(session, window)| {
                    format!("{} / {}", session.name, window.navigation_title())
                }),
            SelectionTarget::Pane(pane_id) => self
                .inventory
                .parent_for_pane(pane_id)
                .zip(self.inventory.pane(pane_id))
                .map(|((session, window), pane)| {
                    format!(
                        "{} / {} / {}",
                        session.name,
                        window.navigation_title(),
                        pane.navigation_title()
                    )
                }),
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

    pub fn focus_help_summary(&self) -> String {
        match self.focus {
            Focus::Sidebar => match self.selection.as_ref() {
                Some(SelectionTarget::Session(_)) => {
                    "Sidebar focus. Enter folds the session row. f and i use the target pane."
                        .to_string()
                }
                Some(SelectionTarget::Window(_)) => {
                    "Sidebar focus. Enter and f act on the resolved target pane for this window."
                        .to_string()
                }
                Some(SelectionTarget::Pane(_)) => {
                    "Sidebar focus. Enter and f act on the selected pane. j and k stay here."
                        .to_string()
                }
                None => "Sidebar focus. Use j and k to move, then Enter or i to act.".to_string(),
            },
            Focus::Preview => {
                "Details focus. Read recent output, PR state, and target-pane guidance here."
                    .to_string()
            }
            Focus::Input => {
                if self.mode == Mode::Input {
                    "Compose focus. Type your instruction here. Enter sends and Ctrl+J adds a newline."
                        .to_string()
                } else if self.selected_actionable_pane().is_some() {
                    "Compose focus. Press Enter or i to start composing for the target pane."
                        .to_string()
                } else {
                    "Compose focus. Select an agent row first so Foreman knows where to send input."
                        .to_string()
                }
            }
        }
    }

    fn filter_search_entries(&self, entries: Vec<VisibleTargetEntry>) -> Vec<VisibleTargetEntry> {
        let Some(query) = self.search_query().map(str::trim) else {
            return entries;
        };
        if query.is_empty() {
            return entries;
        }

        let needle = query.to_ascii_lowercase();
        entries
            .into_iter()
            .filter(|entry| {
                self.target_label(&entry.target)
                    .to_ascii_lowercase()
                    .contains(&needle)
            })
            .collect()
    }

    fn build_base_visible_entries(&self) -> Vec<VisibleTargetEntry> {
        let mut entries = Vec::new();

        for session in self.inventory.sorted_sessions(self.sort_mode) {
            if !session.is_visible(&self.filters) {
                continue;
            }

            let visible_windows = session.visible_windows(&self.filters, self.sort_mode);
            let session_visible_panes = visible_windows
                .iter()
                .flat_map(|window| window.visible_panes(&self.filters, self.sort_mode))
                .collect::<Vec<_>>();

            entries.push(VisibleTargetEntry {
                target: SelectionTarget::Session(session.id.clone()),
                actionable_pane_id: session_visible_panes.first().map(|pane| pane.id.clone()),
                actionable_workspace_path: session_visible_panes
                    .first()
                    .and_then(|pane| pane.working_dir.clone()),
                aggregate_workspace_path: unique_workspace_path(
                    session
                        .windows
                        .iter()
                        .flat_map(|window| window.panes.iter())
                        .filter_map(|pane| pane.working_dir.clone()),
                ),
                sidebar: SidebarRowKind::Session {
                    name: session.name.clone(),
                    collapsed: self.collapsed_sessions.contains(&session.id),
                    rank: session.attention_rank(),
                    visible_windows: visible_windows.len(),
                    visible_panes: session_visible_panes.len(),
                    harnesses: harness_summary_for_panes(session_visible_panes.iter().copied()),
                },
            });

            if self.collapsed_sessions.contains(&session.id) {
                continue;
            }

            for window in visible_windows {
                let visible_panes = window.visible_panes(&self.filters, self.sort_mode);
                entries.push(VisibleTargetEntry {
                    target: SelectionTarget::Window(window.id.clone()),
                    actionable_pane_id: visible_panes.first().map(|pane| pane.id.clone()),
                    actionable_workspace_path: visible_panes
                        .first()
                        .and_then(|pane| pane.working_dir.clone()),
                    aggregate_workspace_path: unique_workspace_path(
                        window
                            .panes
                            .iter()
                            .filter_map(|pane| pane.working_dir.clone()),
                    ),
                    sidebar: SidebarRowKind::Window {
                        name: window.navigation_title(),
                        rank: window.attention_rank(),
                        visible_panes: visible_panes.len(),
                        harnesses: harness_summary_for_panes(visible_panes.iter().copied()),
                    },
                });

                for pane in visible_panes {
                    entries.push(VisibleTargetEntry {
                        target: SelectionTarget::Pane(pane.id.clone()),
                        actionable_pane_id: Some(pane.id.clone()),
                        actionable_workspace_path: pane.working_dir.clone(),
                        aggregate_workspace_path: pane.working_dir.clone(),
                        sidebar: SidebarRowKind::Pane {
                            navigation_title: pane.navigation_title(),
                            status: pane.agent.as_ref().map(|agent| agent.status),
                            harness: pane.harness_kind(),
                            is_agent: pane.is_agent(),
                        },
                    });
                }
            }
        }

        entries
    }
}

fn session_cmp(left: &Session, right: &Session, sort_mode: SortMode) -> Ordering {
    match sort_mode {
        SortMode::Stable => left
            .name
            .cmp(&right.name)
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
        SortMode::Stable => left
            .name
            .cmp(&right.name)
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
        SortMode::Stable => left
            .title
            .cmp(&right.title)
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
        SortMode::AttentionFirst => left
            .attention_rank()
            .cmp(&right.attention_rank())
            .then_with(|| right.recent_activity().cmp(&left.recent_activity()))
            .then_with(|| left.title.cmp(&right.title))
            .then_with(|| left.id.as_str().cmp(right.id.as_str())),
    }
}

fn is_generic_window_name(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "" | "sh" | "zsh" | "bash" | "fish" | "nu" | "shell"
    )
}

fn build_flash_targets(targets: &[SelectionTarget]) -> Vec<FlashTarget> {
    let width = flash_label_width(targets.len());
    targets
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, target)| FlashTarget {
            label: flash_label_for_index(index, width),
            target,
        })
        .collect()
}

fn harness_summary_for_panes<'a>(
    panes: impl IntoIterator<Item = &'a Pane>,
) -> SidebarHarnessSummary {
    let mut harnesses = BTreeSet::new();
    let mut saw_shell = false;

    for pane in panes {
        match pane.harness_kind() {
            Some(harness) => {
                harnesses.insert(harness);
            }
            None => saw_shell = true,
        }
    }

    SidebarHarnessSummary {
        harnesses: harnesses.into_iter().collect(),
        saw_shell,
    }
}

fn unique_workspace_path(paths: impl IntoIterator<Item = PathBuf>) -> Option<PathBuf> {
    let mut paths = paths.into_iter();
    let first = paths.next()?;
    if paths.any(|path| path != first) {
        return None;
    }
    Some(first)
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

#[cfg(test)]
mod tests {
    use super::{AppState, SearchState, SelectionTarget, SortMode};
    use crate::app::{
        inventory, AgentStatus, HarnessKind, PaneBuilder, SessionBuilder, WindowBuilder,
    };
    use std::path::PathBuf;

    #[test]
    fn window_navigation_title_falls_back_from_generic_shell_name() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0").name("sh").pane(
                PaneBuilder::agent("alpha:pane", HarnessKind::ClaudeCode).working_dir("/tmp/alpha"),
            ),
        )]);

        let window = inventory
            .window(&"alpha:0".into())
            .expect("window should exist");
        assert_eq!(window.navigation_title(), "alpha");
    }

    #[test]
    fn selection_breadcrumb_includes_session_window_and_pane_titles() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0").name("agents").pane(
                PaneBuilder::agent("alpha:pane", HarnessKind::ClaudeCode)
                    .title("main")
                    .working_dir("/tmp/foreman"),
            ),
        )]);

        let mut state = AppState::with_inventory(inventory);
        state.selection = Some(SelectionTarget::Pane("alpha:pane".into()));

        assert_eq!(
            state.selection_breadcrumb().as_deref(),
            Some("alpha / agents / foreman")
        );
    }

    #[test]
    fn stable_sort_stays_stable_across_status_changes() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0")
                .name("agents")
                .pane(
                    PaneBuilder::agent("alpha:zeta", HarnessKind::ClaudeCode)
                        .title("zeta")
                        .status(AgentStatus::Error)
                        .activity_score(100),
                )
                .pane(
                    PaneBuilder::agent("alpha:alpha", HarnessKind::ClaudeCode)
                        .title("alpha")
                        .status(AgentStatus::Idle)
                        .activity_score(1),
                ),
        )]);

        let window = inventory
            .window(&"alpha:0".into())
            .expect("window should exist");
        let panes = window.visible_panes(&Default::default(), SortMode::Stable);

        assert_eq!(panes[0].id.as_str(), "alpha:alpha");
        assert_eq!(panes[1].id.as_str(), "alpha:zeta");
    }

    #[test]
    fn attention_sort_uses_recent_activity_as_tiebreaker() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0")
                .name("agents")
                .pane(
                    PaneBuilder::agent("alpha:quiet", HarnessKind::ClaudeCode)
                        .title("quiet")
                        .status(AgentStatus::Working)
                        .activity_score(4),
                )
                .pane(
                    PaneBuilder::agent("alpha:busy", HarnessKind::ClaudeCode)
                        .title("busy")
                        .status(AgentStatus::Working)
                        .activity_score(9),
                ),
        )]);

        let window = inventory
            .window(&"alpha:0".into())
            .expect("window should exist");
        let panes = window.visible_panes(&Default::default(), SortMode::AttentionFirst);

        assert_eq!(panes[0].id.as_str(), "alpha:busy");
        assert_eq!(panes[1].id.as_str(), "alpha:quiet");
    }

    #[test]
    fn rebuild_visible_state_applies_search_filter_to_cached_entries() {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:0")
                    .name("agents")
                    .pane(PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:0")
                    .name("agents")
                    .pane(PaneBuilder::agent("beta:codex", HarnessKind::CodexCli)),
            ),
        ]);

        let mut state = AppState::with_inventory(inventory);
        let mut search = SearchState::new(state.selection.clone());
        search.draft.text = "codex".to_string();
        state.search = Some(search);
        state.rebuild_visible_state();

        assert_eq!(
            state.visible_targets(),
            vec![SelectionTarget::Pane("beta:codex".into())]
        );
        assert_eq!(state.visible_target_count(), 1);
    }

    #[test]
    fn selected_actionable_pane_uses_cached_first_visible_pane() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0")
                .name("agents")
                .pane(
                    PaneBuilder::agent("alpha:idle", HarnessKind::ClaudeCode)
                        .title("idle")
                        .status(AgentStatus::Idle)
                        .activity_score(10),
                )
                .pane(
                    PaneBuilder::agent("alpha:error", HarnessKind::ClaudeCode)
                        .title("error")
                        .status(AgentStatus::Error)
                        .activity_score(10),
                ),
        )]);

        let mut state = AppState::with_inventory(inventory);
        state.sort_mode = SortMode::AttentionFirst;
        state.rebuild_visible_state();

        state.selection = Some(SelectionTarget::Session("alpha".into()));
        assert_eq!(
            state
                .selected_actionable_pane_id()
                .as_ref()
                .map(|pane| pane.as_str()),
            Some("alpha:error")
        );

        state.selection = Some(SelectionTarget::Window("alpha:0".into()));
        assert_eq!(
            state
                .selected_actionable_pane_id()
                .as_ref()
                .map(|pane| pane.as_str()),
            Some("alpha:error")
        );
    }

    #[test]
    fn selected_workspace_path_follows_actionable_pane_for_session_and_window_rows() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:0")
                .name("agents")
                .pane(
                    PaneBuilder::agent("alpha:attention", HarnessKind::ClaudeCode)
                        .title("attention")
                        .working_dir("/tmp/actionable")
                        .status(AgentStatus::NeedsAttention)
                        .activity_score(80),
                )
                .pane(
                    PaneBuilder::agent("alpha:helper", HarnessKind::ClaudeCode)
                        .title("helper")
                        .working_dir("/tmp/helper")
                        .status(AgentStatus::Idle)
                        .activity_score(10),
                ),
        )]);

        let mut state = AppState::with_inventory(inventory);
        state.sort_mode = SortMode::AttentionFirst;
        state.rebuild_visible_state();

        state.selection = Some(SelectionTarget::Session("alpha".into()));
        assert_eq!(
            state.selected_workspace_path(),
            Some(PathBuf::from("/tmp/actionable"))
        );
        let session_entry = state
            .selected_visible_entry()
            .expect("session entry should exist");
        assert_eq!(session_entry.aggregate_workspace_path, None);

        state.selection = Some(SelectionTarget::Window("alpha:0".into()));
        assert_eq!(
            state.selected_workspace_path(),
            Some(PathBuf::from("/tmp/actionable"))
        );
        let window_entry = state
            .selected_visible_entry()
            .expect("window entry should exist");
        assert_eq!(window_entry.aggregate_workspace_path, None);
    }

    #[test]
    fn refresh_priority_pane_ids_include_viewport_selected_and_attention_rows() {
        let inventory = inventory([
            SessionBuilder::new("alpha").window(
                WindowBuilder::new("alpha:0")
                    .name("agents")
                    .pane(
                        PaneBuilder::agent("alpha:error", HarnessKind::ClaudeCode)
                            .title("error")
                            .status(AgentStatus::Error),
                    )
                    .pane(
                        PaneBuilder::agent("alpha:idle", HarnessKind::ClaudeCode)
                            .title("idle")
                            .status(AgentStatus::Idle),
                    ),
            ),
            SessionBuilder::new("beta").window(
                WindowBuilder::new("beta:0").name("agents").pane(
                    PaneBuilder::agent("beta:idle", HarnessKind::CodexCli)
                        .title("beta")
                        .status(AgentStatus::Idle),
                ),
            ),
        ]);

        let mut state = AppState::with_inventory(inventory);
        state.sort_mode = SortMode::AttentionFirst;
        state.rebuild_visible_state();
        state.set_sidebar_viewport_rows(2);
        state.selection = Some(SelectionTarget::Pane("beta:idle".into()));
        state.reconcile_sidebar_scroll();

        let priority = state.refresh_priority_pane_ids();

        assert!(priority.contains(&"beta:idle".into()));
        assert!(priority.contains(&"alpha:error".into()));
    }
}
