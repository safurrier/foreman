use super::{status_from_hints, CompatibilityObservation, StatusHints};
use crate::app::AgentStatus;

const RECOGNITION_TOKENS: &[&str] = &["opencode"];
const STATUS_HINTS: StatusHints = StatusHints {
    attention: &[
        "waiting for your input",
        "needs attention",
        "approve",
        "confirm",
    ],
    error: &["error", "failed", "panic", "traceback", "exception"],
    working: &[
        "working",
        "running",
        "searching",
        "reading",
        "writing",
        "editing",
        "analyzing",
        "processing",
    ],
    idle: &["ready", "idle", "awaiting task", "waiting for task", "done"],
};

pub(crate) fn recognition_tokens() -> &'static [&'static str] {
    RECOGNITION_TOKENS
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_hints(observation, STATUS_HINTS)
}
