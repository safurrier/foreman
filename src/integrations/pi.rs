use super::native;
use super::{matches_any, status_from_hints, CompatibilityObservation, StatusHints};
use crate::app::{AgentStatus, HarnessKind, Inventory};

const RECOGNITION_TOKENS: &[&str] = &["\npi ", "pi loop", "pi agent", "pi-coding-agent"];
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
        "thinking",
    ],
    idle: &["ready", "idle", "awaiting task", "waiting for task", "done"],
};

pub use native::{
    FileNativeSignalSource as FilePiNativeSignalSource,
    NativeOverlaySummary as PiNativeOverlaySummary, NativeSignalSource as PiNativeSignalSource,
};

pub(crate) fn recognizes(observation: CompatibilityObservation<'_>) -> bool {
    if observation
        .current_command
        .and_then(|command| command.split_whitespace().next())
        .map(command_basename)
        .is_some_and(|command| command == "pi")
    {
        return true;
    }

    matches_any(observation, RECOGNITION_TOKENS)
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_hints(observation, STATUS_HINTS)
}

pub fn apply_native_signals<S: PiNativeSignalSource>(
    inventory: &mut Inventory,
    source: &S,
) -> PiNativeOverlaySummary {
    native::apply_native_signals(inventory, HarnessKind::Pi, source)
}

pub(crate) fn compatibility_fallback_summary(
    inventory: &Inventory,
    warn_missing_native: bool,
) -> PiNativeOverlaySummary {
    native::compatibility_fallback_summary(
        inventory,
        HarnessKind::Pi,
        warn_missing_native.then_some(
            "pi native preference requested but no native signal source was configured".to_string(),
        ),
    )
}

fn command_basename(command: &str) -> &str {
    command.rsplit('/').next().unwrap_or(command)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_native_signals, compatibility_fallback_summary, compatibility_status,
        FilePiNativeSignalSource,
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
    fn compatibility_status_maps_pi_phrases() {
        assert_eq!(
            compatibility_status(observation(Some("pi"), "shell", "Pi is thinking")),
            AgentStatus::Working
        );
        assert_eq!(
            compatibility_status(observation(
                Some("pi"),
                "shell",
                "Pi waiting for your input",
            )),
            AgentStatus::NeedsAttention
        );
        assert_eq!(
            compatibility_status(observation(Some("pi"), "shell", "Pi ready")),
            AgentStatus::Idle
        );
    }

    #[test]
    fn file_native_source_applies_precedence_over_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%1.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal file should exist");

        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
        );

        let agent = inventory
            .pane(&"%1".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn missing_native_signal_falls_back_to_compatibility() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
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
                PaneBuilder::agent("%1", HarnessKind::Pi)
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
