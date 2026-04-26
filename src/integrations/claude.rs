use super::native;
use super::{status_from_recent_preview_lines, CompatibilityObservation, StatusHints};
use crate::app::{AgentStatus, HarnessKind, Inventory};

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
    idle: &[
        "ready",
        "idle",
        "awaiting task",
        "waiting for task",
        "done",
        "new task?",
        "/clear to save",
        "-- insert --",
        "bypass permissions on",
    ],
};

const RECENT_STATUS_LINES: usize = 24;

pub use native::{
    FileNativeSignalSource as FileClaudeNativeSignalSource,
    NativeOverlaySummary as ClaudeNativeOverlaySummary,
    NativeSignalSource as ClaudeNativeSignalSource,
};

pub(crate) fn recognition_tokens() -> &'static [&'static str] {
    RECOGNITION_TOKENS
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_recent_preview_lines(observation, STATUS_HINTS, RECENT_STATUS_LINES)
}

pub fn apply_native_signals<S: ClaudeNativeSignalSource>(
    inventory: &mut Inventory,
    source: &S,
) -> ClaudeNativeOverlaySummary {
    native::apply_native_signals(inventory, HarnessKind::ClaudeCode, source)
}

pub(crate) fn compatibility_fallback_summary(
    inventory: &Inventory,
    warn_missing_native: bool,
) -> ClaudeNativeOverlaySummary {
    native::compatibility_fallback_summary(
        inventory,
        HarnessKind::ClaudeCode,
        warn_missing_native.then_some(
            "claude native preference requested but no native signal source was configured"
                .to_string(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        apply_native_signals, compatibility_fallback_summary, compatibility_status,
        FileClaudeNativeSignalSource,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, PaneBuilder, SessionBuilder,
        WindowBuilder,
    };
    use crate::integrations::CompatibilityObservation;
    use tempfile::tempdir;

    fn observation<'a>(
        current_command: Option<&'a str>,
        title: &'a str,
        preview: &'a str,
    ) -> CompatibilityObservation<'a> {
        CompatibilityObservation::new(current_command, title, preview)
    }

    #[test]
    fn compatibility_status_maps_claude_phrases() {
        assert_eq!(
            compatibility_status(observation(Some("claude"), "shell", "Claude is thinking")),
            AgentStatus::Working
        );
        assert_eq!(
            compatibility_status(observation(
                Some("claude"),
                "shell",
                "Claude waiting for your input",
            )),
            AgentStatus::NeedsAttention
        );
        assert_eq!(
            compatibility_status(observation(Some("claude"), "shell", "Claude ready")),
            AgentStatus::Idle
        );

        assert_eq!(
            compatibility_status(observation(
                Some("claude.exe"),
                "✳ Claude Code",
                "PostToolUse:Edit hook error\nFailed with non-blocking status code\n────────────────────────\n-- INSERT -- ⏵⏵ bypass permissions on (shift+tab to cycle) · new task? /clear to save",
            )),
            AgentStatus::Idle
        );
        assert_eq!(
            compatibility_status(observation(
                Some("claude.exe"),
                "✳ Claude Code",
                "All good\n────────────────────────\nError: current hook failed",
            )),
            AgentStatus::Error
        );
    }

    #[test]
    fn file_native_source_applies_precedence_over_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%1.json"),
            r#"{"status":"needs_attention","activity_score":91}"#,
        )
        .expect("signal file should exist");

        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FileClaudeNativeSignalSource::new(temp_dir.path().to_path_buf()),
        );

        let agent = inventory
            .pane(&"%1".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::NeedsAttention);
        assert_eq!(agent.activity_score, 91);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn missing_native_signal_falls_back_to_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FileClaudeNativeSignalSource::new(temp_dir.path().to_path_buf()),
        );

        let agent = inventory
            .pane(&"%1".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(agent.status, AgentStatus::Working);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.fallback_to_compatibility, 1);
    }

    #[test]
    fn missing_native_source_can_report_compatibility_fallback() {
        let inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::ClaudeCode)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = compatibility_fallback_summary(&inventory, true);

        assert_eq!(summary.applied, 0);
        assert_eq!(summary.fallback_to_compatibility, 1);
        assert_eq!(summary.warnings.len(), 1);
    }
}
