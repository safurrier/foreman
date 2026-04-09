use super::{status_from_hints, CompatibilityObservation, StatusHints};
use crate::app::AgentStatus;

const RECOGNITION_TOKENS: &[&str] = &["claude", "claude code"];
const STATUS_HINTS: StatusHints = StatusHints {
    attention: &[
        "waiting for your input",
        "waiting on you",
        "needs attention",
        "approve",
        "approval",
        "press enter to continue",
        "confirm to continue",
    ],
    error: &["error", "failed", "panic", "traceback", "exception"],
    working: &[
        "thinking",
        "working",
        "applying patch",
        "running",
        "searching",
        "reading",
        "writing",
        "editing",
        "analyzing",
        "updating",
    ],
    idle: &["ready", "idle", "awaiting task", "waiting for task", "done"],
};

pub(crate) fn recognition_tokens() -> &'static [&'static str] {
    RECOGNITION_TOKENS
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_hints(observation, STATUS_HINTS)
}
