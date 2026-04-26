mod claude;
mod claude_hook;
mod codex;
mod codex_hook;
mod gemini;
mod native;
mod opencode;
mod pi;
mod pi_hook;

use crate::app::{AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory};
use crate::config::IntegrationPreference;
pub use claude::{
    apply_native_signals as apply_claude_native_signals, ClaudeNativeOverlaySummary,
    FileClaudeNativeSignalSource,
};
pub use claude_hook::{
    bridge_claude_hook_input, ClaudeHookBridgeError, ClaudeHookBridgeRequest, ClaudeHookEventKind,
};
pub use codex::{
    apply_native_signals as apply_codex_native_signals, CodexNativeOverlaySummary,
    FileCodexNativeSignalSource,
};
pub use codex_hook::{
    bridge_codex_hook_input, CodexHookBridgeError, CodexHookBridgeRequest, CodexHookEventKind,
};
pub use pi::{
    apply_native_signals as apply_pi_native_signals, FilePiNativeSignalSource,
    PiNativeOverlaySummary,
};
pub use pi_hook::{bridge_pi_event, PiHookBridgeError, PiHookBridgeRequest, PiHookEventKind};
use std::path::Path;

const WORKING_STATUS_DEBOUNCE_POLLS: u8 = 2;
const SHELL_COMMANDS: &[&str] = &[
    "ash", "bash", "csh", "dash", "fish", "ksh", "nu", "pwsh", "sh", "tcsh", "xonsh", "zsh",
];
const EDITOR_COMMANDS: &[&str] = &["emacs", "hx", "nano", "nvim", "vi", "vim"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityObservation<'a> {
    pub current_command: Option<&'a str>,
    pub title: &'a str,
    pub preview: &'a str,
}

impl<'a> CompatibilityObservation<'a> {
    pub fn new(current_command: Option<&'a str>, title: &'a str, preview: &'a str) -> Self {
        Self {
            current_command,
            title,
            preview,
        }
    }

    fn haystack(self) -> String {
        format!(
            "{}\n{}\n{}",
            self.current_command.unwrap_or_default(),
            self.title,
            self.preview
        )
        .to_ascii_lowercase()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatusHints {
    pub attention: &'static [&'static str],
    pub error: &'static [&'static str],
    pub working: &'static [&'static str],
    pub idle: &'static [&'static str],
}

pub fn compatibility_snapshot(observation: CompatibilityObservation<'_>) -> Option<AgentSnapshot> {
    let harness = recognize_harness(
        observation.current_command,
        observation.title,
        observation.preview,
    )?;
    let observed_status = compatibility_status(harness, observation);

    Some(AgentSnapshot {
        harness,
        status: observed_status,
        observed_status,
        integration_mode: IntegrationMode::Compatibility,
        activity_score: activity_score_for_status(observed_status),
        debounce_ticks: 0,
    })
}

pub fn stabilize_inventory(previous: &Inventory, next: &mut Inventory) {
    for session in &mut next.sessions {
        for window in &mut session.windows {
            for pane in &mut window.panes {
                let Some(current) = pane.agent.as_mut() else {
                    continue;
                };
                let Some(previous_agent) =
                    previous.pane(&pane.id).and_then(|pane| pane.agent.as_ref())
                else {
                    continue;
                };

                debounce_snapshot(previous_agent, current);
            }
        }
    }
}

pub fn apply_configured_claude_signals(
    inventory: &mut Inventory,
    native_dir: Option<&Path>,
    preference: IntegrationPreference,
) -> ClaudeNativeOverlaySummary {
    match preference {
        IntegrationPreference::Compatibility => ClaudeNativeOverlaySummary::default(),
        IntegrationPreference::Auto | IntegrationPreference::Native => {
            if let Some(dir) = native_dir {
                return claude::apply_native_signals(
                    inventory,
                    &FileClaudeNativeSignalSource::new(dir.to_path_buf()),
                );
            }

            claude::compatibility_fallback_summary(
                inventory,
                matches!(preference, IntegrationPreference::Native),
            )
        }
    }
}

pub fn apply_configured_codex_signals(
    inventory: &mut Inventory,
    native_dir: Option<&Path>,
    preference: IntegrationPreference,
) -> CodexNativeOverlaySummary {
    match preference {
        IntegrationPreference::Compatibility => CodexNativeOverlaySummary::default(),
        IntegrationPreference::Auto | IntegrationPreference::Native => {
            if let Some(dir) = native_dir {
                return codex::apply_native_signals(
                    inventory,
                    &FileCodexNativeSignalSource::new(dir.to_path_buf()),
                );
            }

            codex::compatibility_fallback_summary(
                inventory,
                matches!(preference, IntegrationPreference::Native),
            )
        }
    }
}

pub fn apply_configured_pi_signals(
    inventory: &mut Inventory,
    native_dir: Option<&Path>,
    preference: IntegrationPreference,
) -> PiNativeOverlaySummary {
    match preference {
        IntegrationPreference::Compatibility => PiNativeOverlaySummary::default(),
        IntegrationPreference::Auto | IntegrationPreference::Native => {
            if let Some(dir) = native_dir {
                return pi::apply_native_signals(
                    inventory,
                    &FilePiNativeSignalSource::new(dir.to_path_buf()),
                );
            }

            pi::compatibility_fallback_summary(
                inventory,
                matches!(preference, IntegrationPreference::Native),
            )
        }
    }
}

pub fn recognize_harness(
    current_command: Option<&str>,
    title: &str,
    preview: &str,
) -> Option<HarnessKind> {
    let observation = CompatibilityObservation::new(current_command, title, preview);

    if is_foreman_surface(observation) {
        return None;
    }

    if is_shell_surface(observation) || is_editor_surface(observation) {
        return None;
    }

    if matches_any(observation, claude::recognition_tokens()) {
        return Some(HarnessKind::ClaudeCode);
    }

    if matches_any(observation, codex::recognition_tokens()) {
        return Some(HarnessKind::CodexCli);
    }

    if pi::recognizes(observation) {
        return Some(HarnessKind::Pi);
    }

    if matches_any(observation, gemini::recognition_tokens()) {
        return Some(HarnessKind::GeminiCli);
    }

    if matches_any(observation, opencode::recognition_tokens()) {
        return Some(HarnessKind::OpenCode);
    }

    None
}

pub fn compatibility_status(
    harness: HarnessKind,
    observation: CompatibilityObservation<'_>,
) -> AgentStatus {
    match harness {
        HarnessKind::ClaudeCode => claude::compatibility_status(observation),
        HarnessKind::CodexCli => codex::compatibility_status(observation),
        HarnessKind::Pi => pi::compatibility_status(observation),
        HarnessKind::GeminiCli => gemini::compatibility_status(observation),
        HarnessKind::OpenCode => opencode::compatibility_status(observation),
    }
}

pub(crate) fn status_from_hints(
    observation: CompatibilityObservation<'_>,
    hints: StatusHints,
) -> AgentStatus {
    status_from_text(&observation.haystack(), hints)
}

pub(crate) fn status_from_recent_preview_lines(
    observation: CompatibilityObservation<'_>,
    hints: StatusHints,
    max_lines: usize,
) -> AgentStatus {
    for line in observation
        .preview
        .lines()
        .rev()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(max_lines)
    {
        let status = status_from_text(&line.to_ascii_lowercase(), hints);
        if status != AgentStatus::Unknown {
            return status;
        }
    }

    let tail = observation
        .preview
        .lines()
        .rev()
        .take(max_lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n");
    let haystack = format!(
        "{}\n{}\n{}",
        observation.current_command.unwrap_or_default(),
        observation.title,
        tail
    )
    .to_ascii_lowercase();
    status_from_text(&haystack, hints)
}

fn status_from_text(haystack: &str, hints: StatusHints) -> AgentStatus {
    if contains_any(haystack, hints.attention) {
        return AgentStatus::NeedsAttention;
    }

    if contains_any(haystack, hints.error) {
        return AgentStatus::Error;
    }

    if contains_any(haystack, hints.working) {
        return AgentStatus::Working;
    }

    if contains_any(haystack, hints.idle) {
        return AgentStatus::Idle;
    }

    AgentStatus::Unknown
}

fn contains_any<T>(haystack: &str, needles: impl IntoIterator<Item = T>) -> bool
where
    T: AsRef<str>,
{
    needles
        .into_iter()
        .any(|needle| haystack.contains(&needle.as_ref().to_ascii_lowercase()))
}

fn debounce_snapshot(previous: &AgentSnapshot, current: &mut AgentSnapshot) {
    if previous.harness != current.harness {
        current.status = current.observed_status;
        current.debounce_ticks = 0;
        return;
    }

    if previous.status == AgentStatus::Working
        && matches!(
            current.observed_status,
            AgentStatus::Idle | AgentStatus::Unknown
        )
    {
        let next_ticks = if matches!(
            previous.observed_status,
            AgentStatus::Idle | AgentStatus::Unknown
        ) {
            previous.debounce_ticks.saturating_add(1)
        } else {
            1
        };

        if next_ticks < WORKING_STATUS_DEBOUNCE_POLLS {
            current.status = AgentStatus::Working;
            current.debounce_ticks = next_ticks;
            current.activity_score = previous
                .activity_score
                .max(activity_score_for_status(AgentStatus::Working));
            return;
        }
    }

    current.status = current.observed_status;
    current.debounce_ticks = 0;
    current.activity_score = activity_score_for_status(current.status);
}

fn activity_score_for_status(status: AgentStatus) -> u64 {
    match status {
        AgentStatus::Working => 100,
        AgentStatus::NeedsAttention => 80,
        AgentStatus::Error => 60,
        AgentStatus::Idle => 30,
        AgentStatus::Unknown => 0,
    }
}

fn is_foreman_surface(observation: CompatibilityObservation<'_>) -> bool {
    let preview = observation.preview.to_ascii_lowercase();

    foreground_command_basename(observation.current_command)
        .is_some_and(|command| command == "foreman")
        || (preview.contains("foreman | ")
            && preview.contains("targets")
            && preview.contains("compose"))
}

fn is_shell_surface(observation: CompatibilityObservation<'_>) -> bool {
    foreground_command_basename(observation.current_command)
        .is_some_and(|command| SHELL_COMMANDS.contains(&command))
}

fn is_editor_surface(observation: CompatibilityObservation<'_>) -> bool {
    foreground_command_basename(observation.current_command)
        .is_some_and(|command| EDITOR_COMMANDS.contains(&command))
}

fn foreground_command_basename(current_command: Option<&str>) -> Option<&str> {
    current_command
        .and_then(|command| command.split_whitespace().next())
        .map(command_basename)
}

fn command_basename(command: &str) -> &str {
    command.rsplit('/').next().unwrap_or(command)
}

pub(crate) fn matches_any<T>(
    observation: CompatibilityObservation<'_>,
    needles: impl IntoIterator<Item = T>,
) -> bool
where
    T: AsRef<str>,
{
    let haystack = observation.haystack();
    contains_any(&haystack, needles)
}

#[cfg(test)]
mod tests {
    use super::{
        compatibility_snapshot, compatibility_status, recognize_harness, stabilize_inventory,
        CompatibilityObservation,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, Inventory, PaneBuilder,
        SessionBuilder, WindowBuilder,
    };

    fn observation<'a>(
        current_command: Option<&'a str>,
        title: &'a str,
        preview: &'a str,
    ) -> CompatibilityObservation<'a> {
        CompatibilityObservation::new(current_command, title, preview)
    }

    fn single_pane_inventory(
        pane_id: &str,
        harness: HarnessKind,
        status: AgentStatus,
        observed_status: AgentStatus,
        debounce_ticks: u8,
    ) -> Inventory {
        inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent(pane_id, harness)
                    .status(status)
                    .observed_status(observed_status)
                    .debounce_ticks(debounce_ticks),
            ),
        )])
    }

    #[test]
    fn recognizes_supported_harnesses_from_command_title_and_preview() {
        assert_eq!(
            recognize_harness(Some("claude"), "shell", ""),
            Some(HarnessKind::ClaudeCode)
        );
        assert_eq!(
            recognize_harness(None, "codex-main", ""),
            Some(HarnessKind::CodexCli)
        );
        assert_eq!(
            recognize_harness(Some("pi"), "shell", ""),
            Some(HarnessKind::Pi)
        );
        assert_eq!(
            recognize_harness(None, "shell", "Gemini CLI ready"),
            Some(HarnessKind::GeminiCli)
        );
        assert_eq!(
            recognize_harness(None, "shell", "OpenCode interactive shell"),
            Some(HarnessKind::OpenCode)
        );
        assert_eq!(
            recognize_harness(Some("node"), "shell", "Codex CLI waiting for your input"),
            Some(HarnessKind::CodexCli)
        );
    }

    #[test]
    fn returns_none_for_unrecognized_panes() {
        assert_eq!(recognize_harness(Some("zsh"), "notes", "plain shell"), None);
    }

    #[test]
    fn returns_none_for_foreman_dashboard_surface() {
        assert_eq!(
            recognize_harness(
                Some("sh"),
                "dashboard",
                "Foreman | NORMAL | cpu=? mem=? | 3 targets | pr=NONE | notify=MUTED\n* Targets\nCompose"
            ),
            None
        );
    }

    #[test]
    fn returns_none_for_shell_panes_with_stale_harness_breadcrumbs() {
        assert_eq!(
            recognize_harness(Some("zsh"), "claude", "Claude Code ready for the next task",),
            None
        );
        assert_eq!(
            recognize_harness(Some("bash"), "shell", "Codex CLI waiting for your input",),
            None
        );
    }

    #[test]
    fn returns_none_for_editor_panes_with_stale_harness_breadcrumbs() {
        assert_eq!(
            recognize_harness(Some("nvim"), "notes", "Claude Code ready for the next task",),
            None
        );
        assert_eq!(
            recognize_harness(
                Some("/usr/bin/vim"),
                "scratch",
                "Codex CLI waiting for your input",
            ),
            None
        );
    }

    #[test]
    fn compatibility_snapshot_maps_supported_harness_statuses() {
        let claude = compatibility_snapshot(observation(
            Some("claude"),
            "claude",
            "Claude Code is thinking about the next patch",
        ))
        .expect("snapshot should exist");
        assert_eq!(claude.harness, HarnessKind::ClaudeCode);
        assert_eq!(claude.status, AgentStatus::Working);

        let codex = compatibility_snapshot(observation(
            Some("node"),
            "codex",
            "Codex CLI waiting for your input before continuing",
        ))
        .expect("snapshot should exist");
        assert_eq!(codex.status, AgentStatus::NeedsAttention);

        let pi = compatibility_snapshot(observation(
            Some("pi"),
            "shell",
            "Pi ready for the next task",
        ))
        .expect("snapshot should exist");
        assert_eq!(pi.harness, HarnessKind::Pi);
        assert_eq!(pi.status, AgentStatus::Idle);

        let gemini = compatibility_snapshot(observation(
            Some("python3"),
            "gemini",
            "Gemini CLI ready for the next task",
        ))
        .expect("snapshot should exist");
        assert_eq!(gemini.status, AgentStatus::Idle);

        let opencode = compatibility_snapshot(observation(
            Some("node"),
            "opencode",
            "OpenCode panic: transport failed",
        ))
        .expect("snapshot should exist");
        assert_eq!(opencode.status, AgentStatus::Error);
    }

    #[test]
    fn compatibility_snapshot_drops_shell_panes_with_stale_harness_text() {
        assert!(compatibility_snapshot(observation(
            Some("zsh"),
            "claude",
            "Claude Code is thinking about the next patch",
        ))
        .is_none());
    }

    #[test]
    fn compatibility_snapshot_drops_editor_panes_with_stale_harness_text() {
        assert!(compatibility_snapshot(observation(
            Some("nvim"),
            "notes",
            "Claude Code is thinking about the next patch",
        ))
        .is_none());
    }

    #[test]
    fn compatibility_status_returns_unknown_without_status_cues() {
        let status = compatibility_status(
            HarnessKind::ClaudeCode,
            observation(Some("claude"), "shell", "session attached"),
        );

        assert_eq!(status, AgentStatus::Unknown);
    }

    #[test]
    fn stabilize_inventory_debounces_brief_working_signal_loss() {
        let previous = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Working,
            AgentStatus::Working,
            0,
        );
        let mut next = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Idle,
            AgentStatus::Idle,
            0,
        );

        stabilize_inventory(&previous, &mut next);

        let pane = next
            .pane(&"alpha:claude".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(pane.status, AgentStatus::Working);
        assert_eq!(pane.observed_status, AgentStatus::Idle);
        assert_eq!(pane.debounce_ticks, 1);
    }

    #[test]
    fn stabilize_inventory_allows_idle_after_second_consecutive_loss() {
        let previous = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Working,
            AgentStatus::Idle,
            1,
        );
        let mut next = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Idle,
            AgentStatus::Idle,
            0,
        );

        stabilize_inventory(&previous, &mut next);

        let pane = next
            .pane(&"alpha:claude".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(pane.status, AgentStatus::Idle);
        assert_eq!(pane.debounce_ticks, 0);
    }

    #[test]
    fn stabilize_inventory_surfaces_attention_immediately() {
        let previous = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::Working,
            AgentStatus::Working,
            0,
        );
        let mut next = single_pane_inventory(
            "alpha:claude",
            HarnessKind::ClaudeCode,
            AgentStatus::NeedsAttention,
            AgentStatus::NeedsAttention,
            0,
        );

        stabilize_inventory(&previous, &mut next);

        let pane = next
            .pane(&"alpha:claude".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(pane.status, AgentStatus::NeedsAttention);
        assert_eq!(pane.debounce_ticks, 0);
    }

    #[test]
    fn stabilize_inventory_debounces_brief_native_working_signal_loss() {
        let previous = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .integration_mode(IntegrationMode::Native)
                    .status(AgentStatus::Working)
                    .observed_status(AgentStatus::Working),
            ),
        )]);
        let mut next = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .integration_mode(IntegrationMode::Native)
                    .status(AgentStatus::Idle)
                    .observed_status(AgentStatus::Idle),
            ),
        )]);

        stabilize_inventory(&previous, &mut next);

        let pane = next
            .pane(&"alpha:claude".into())
            .expect("pane should exist")
            .agent
            .as_ref()
            .expect("agent should exist");
        assert_eq!(pane.status, AgentStatus::Working);
        assert_eq!(pane.observed_status, AgentStatus::Idle);
        assert_eq!(pane.debounce_ticks, 1);
    }
}
