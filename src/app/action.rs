use crate::app::state::{Focus, Inventory, Mode, SelectionTarget, SessionId, SortMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionDirection {
    Previous,
    Next,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    RequestMode(Mode),
    CancelMode,
    SetFocus(Focus),
    SetSelection(SelectionTarget),
    MoveSelection(SelectionDirection),
    ReplaceInventory(Inventory),
    ToggleShowNonAgentSessions,
    ToggleShowNonAgentPanes,
    ToggleSessionCollapsed(SessionId),
    SetSortMode(SortMode),
    SetSearchQuery(String),
    ClearSearchQuery,
    Noop,
}
