use super::native;
use super::pi_subagents::{
    FilePiSubagentActivitySource, PiSubagentActivity, PiSubagentActivitySource,
};
use super::{
    matches_any, status_from_hints, status_from_hints_explanation, CompatibilityExplanation,
    CompatibilityObservation, StatusHints,
};
use crate::app::{AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory};

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
    recognizes_runtime_identity(observation) || matches_any(observation, RECOGNITION_TOKENS)
}

pub(crate) fn recognizes_runtime_identity(observation: CompatibilityObservation<'_>) -> bool {
    let current_command = observation
        .current_command
        .and_then(|command| command.split_whitespace().next())
        .map(command_basename);

    if current_command.is_some_and(|command| command == "pi") {
        return true;
    }

    observation.title.contains('π') && !current_command.is_some_and(is_shell_or_editor_command)
}

pub(crate) fn compatibility_status(observation: CompatibilityObservation<'_>) -> AgentStatus {
    status_from_hints(observation, STATUS_HINTS)
}

pub(crate) fn compatibility_explanation(
    observation: CompatibilityObservation<'_>,
) -> CompatibilityExplanation {
    status_from_hints_explanation(observation, STATUS_HINTS)
}

pub fn apply_native_signals<S: PiNativeSignalSource>(
    inventory: &mut Inventory,
    source: &S,
) -> PiNativeOverlaySummary {
    apply_native_signals_with_subagents(
        inventory,
        source,
        &FilePiSubagentActivitySource::default_user(),
    )
}

pub(crate) fn apply_native_signals_with_subagents<S, A>(
    inventory: &mut Inventory,
    source: &S,
    subagent_source: &A,
) -> PiNativeOverlaySummary
where
    S: PiNativeSignalSource,
    A: PiSubagentActivitySource,
{
    let mut summary = native::apply_native_signals(inventory, HarnessKind::Pi, source);
    apply_subagent_activity(inventory, subagent_source, &mut summary);
    summary
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

fn is_shell_or_editor_command(command: &str) -> bool {
    matches!(
        command,
        "ash"
            | "bash"
            | "csh"
            | "dash"
            | "fish"
            | "ksh"
            | "nu"
            | "pwsh"
            | "sh"
            | "tcsh"
            | "xonsh"
            | "zsh"
            | "emacs"
            | "hx"
            | "nano"
            | "nvim"
            | "vi"
            | "vim"
    )
}

fn apply_subagent_activity<A>(
    inventory: &mut Inventory,
    subagent_source: &A,
    summary: &mut PiNativeOverlaySummary,
) where
    A: PiSubagentActivitySource,
{
    for session in &mut inventory.sessions {
        for window in &mut session.windows {
            for pane in &mut window.panes {
                let Some(working_dir) = pane.working_dir.as_deref() else {
                    continue;
                };
                let Some(activity) = subagent_source.activity_for_workspace(working_dir) else {
                    continue;
                };
                let was_compatibility_fallback = pane.agent.as_ref().is_some_and(|agent| {
                    agent.harness == HarnessKind::Pi
                        && agent.integration_mode == IntegrationMode::Compatibility
                });
                if merge_subagent_activity(&mut pane.agent, &activity) {
                    pane.activity_unix_millis = activity
                        .last_activity_unix_millis
                        .or(pane.activity_unix_millis);
                    if was_compatibility_fallback
                        && pane
                            .agent
                            .as_ref()
                            .is_some_and(|agent| agent.integration_mode == IntegrationMode::Native)
                    {
                        summary.fallback_to_compatibility =
                            summary.fallback_to_compatibility.saturating_sub(1);
                    }
                    summary.applied += 1;
                }
            }
        }
    }
}

fn merge_subagent_activity(
    agent: &mut Option<AgentSnapshot>,
    activity: &PiSubagentActivity,
) -> bool {
    let Some(snapshot) = agent.as_mut() else {
        return false;
    };
    if snapshot.harness != HarnessKind::Pi {
        return false;
    }

    if !activity.has_active_work() && !activity.needs_attention {
        return false;
    }

    let previous = snapshot.clone();
    if activity.has_active_work() {
        snapshot.active_run_count = Some(activity.active_run_count);
    }

    if activity.needs_attention && snapshot.status != AgentStatus::Error {
        snapshot.status = AgentStatus::NeedsAttention;
        snapshot.observed_status = AgentStatus::NeedsAttention;
        snapshot.integration_mode = IntegrationMode::Native;
        snapshot.activity_score = snapshot.activity_score.max(120);
        return *snapshot != previous;
    }

    if activity.has_active_work()
        && !matches!(
            snapshot.status,
            AgentStatus::NeedsAttention | AgentStatus::Error
        )
    {
        snapshot.status = AgentStatus::Working;
        snapshot.observed_status = AgentStatus::Working;
        snapshot.integration_mode = IntegrationMode::Native;
        snapshot.activity_score = snapshot.activity_score.max(120);
    }

    *snapshot != previous
}

#[cfg(test)]
mod tests {
    use super::{
        apply_native_signals, apply_native_signals_with_subagents, compatibility_fallback_summary,
        compatibility_status, recognizes_runtime_identity, FilePiNativeSignalSource,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, PaneBuilder, SessionBuilder,
        WindowBuilder,
    };
    use crate::integrations::pi_subagents::{PiSubagentActivity, PiSubagentActivitySource};
    use crate::integrations::CompatibilityObservation;
    use std::path::{Path, PathBuf};
    use tempfile::tempdir;

    #[derive(Debug, Clone)]
    struct FakeSubagentActivitySource {
        workspace: PathBuf,
        activity: PiSubagentActivity,
    }

    impl PiSubagentActivitySource for FakeSubagentActivitySource {
        fn activity_for_workspace(&self, working_dir: &Path) -> Option<PiSubagentActivity> {
            (working_dir == self.workspace).then_some(self.activity.clone())
        }
    }

    fn observation<'a>(
        current_command: Option<&'a str>,
        title: &'a str,
        preview: &'a str,
    ) -> CompatibilityObservation<'a> {
        CompatibilityObservation::new(current_command, None, title, preview)
    }

    #[test]
    fn runtime_identity_ignores_stale_pi_titles_on_shells_and_editors() {
        assert!(!recognizes_runtime_identity(observation(
            Some("zsh"),
            "π - old",
            ""
        )));
        assert!(!recognizes_runtime_identity(observation(
            Some("nvim"),
            "π - old",
            ""
        )));
        assert!(recognizes_runtime_identity(observation(
            Some("pi"),
            "π - live",
            ""
        )));
        assert!(recognizes_runtime_identity(observation(
            Some("node"),
            "π - live",
            ""
        )));
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
            .pane(&crate::app::PaneKey::from("%1"))
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
            .pane(&crate::app::PaneKey::from("%1"))
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

    #[test]
    fn active_subagent_activity_promotes_idle_pi_parent_to_working() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let workspace = temp_dir.path().join("workspace");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .working_dir(workspace.to_string_lossy().as_ref())
                    .status(AgentStatus::Idle)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);
        let subagents = FakeSubagentActivitySource {
            workspace: workspace.clone(),
            activity: PiSubagentActivity {
                active_run_count: 2,
                failed_count: 0,
                needs_attention: false,
                last_activity_unix_millis: Some(123),
                runs: Vec::new(),
            },
        };

        let summary = apply_native_signals_with_subagents(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
            &subagents,
        );

        let pane = inventory
            .pane(&crate::app::PaneKey::from("%1"))
            .expect("pane should exist");
        let agent = pane.agent.as_ref().expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::Working);
        assert_eq!(agent.observed_status, AgentStatus::Working);
        assert_eq!(agent.active_run_count, Some(2));
        assert_eq!(pane.activity_unix_millis, Some(123));
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn active_subagent_activity_does_not_downgrade_native_error() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let workspace = temp_dir.path().join("workspace");
        std::fs::write(
            temp_dir.path().join("%1.json"),
            r#"{"status":"error","activity_score":70}"#,
        )
        .expect("signal file should exist");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .working_dir(workspace.to_string_lossy().as_ref())
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);
        let subagents = FakeSubagentActivitySource {
            workspace,
            activity: PiSubagentActivity {
                active_run_count: 1,
                failed_count: 0,
                needs_attention: false,
                last_activity_unix_millis: Some(123),
                runs: Vec::new(),
            },
        };

        let summary = apply_native_signals_with_subagents(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
            &subagents,
        );

        let agent = inventory
            .pane(&crate::app::PaneKey::from("%1"))
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::Error);
        assert_eq!(agent.active_run_count, Some(1));
        assert_eq!(summary.applied, 2);
    }

    #[test]
    fn subagent_activity_for_other_workspace_does_not_promote_parent() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let workspace = temp_dir.path().join("workspace");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .working_dir(workspace.to_string_lossy().as_ref())
                    .status(AgentStatus::Idle)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);
        let subagents = FakeSubagentActivitySource {
            workspace: temp_dir.path().join("other"),
            activity: PiSubagentActivity {
                active_run_count: 1,
                failed_count: 0,
                needs_attention: false,
                last_activity_unix_millis: Some(123),
                runs: Vec::new(),
            },
        };

        let summary = apply_native_signals_with_subagents(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
            &subagents,
        );

        let agent = inventory
            .pane(&crate::app::PaneKey::from("%1"))
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.active_run_count, None);
        assert_eq!(summary.applied, 0);
    }

    #[test]
    fn failed_only_subagent_activity_does_not_report_active_runs() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let workspace = temp_dir.path().join("workspace");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .working_dir(workspace.to_string_lossy().as_ref())
                    .status(AgentStatus::Idle)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);
        let subagents = FakeSubagentActivitySource {
            workspace,
            activity: PiSubagentActivity {
                active_run_count: 0,
                failed_count: 1,
                needs_attention: false,
                last_activity_unix_millis: Some(123),
                runs: Vec::new(),
            },
        };

        let summary = apply_native_signals_with_subagents(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
            &subagents,
        );

        let agent = inventory
            .pane(&crate::app::PaneKey::from("%1"))
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(agent.active_run_count, None);
        assert_eq!(summary.applied, 0);
    }

    #[test]
    fn subagent_attention_promotes_parent_without_downgrading_error_precedence() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let workspace = temp_dir.path().join("workspace");
        let mut inventory = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("%1", HarnessKind::Pi)
                    .working_dir(workspace.to_string_lossy().as_ref())
                    .status(AgentStatus::Idle)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);
        let subagents = FakeSubagentActivitySource {
            workspace,
            activity: PiSubagentActivity {
                active_run_count: 0,
                failed_count: 1,
                needs_attention: true,
                last_activity_unix_millis: Some(123),
                runs: Vec::new(),
            },
        };

        let summary = apply_native_signals_with_subagents(
            &mut inventory,
            &FilePiNativeSignalSource::new(temp_dir.path().to_path_buf()),
            &subagents,
        );

        let agent = inventory
            .pane(&crate::app::PaneKey::from("%1"))
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::NeedsAttention);
        assert_eq!(agent.active_run_count, None);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }
}
