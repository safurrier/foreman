use crate::app::{
    AgentStatus, Inventory, NotificationCooldownKey, NotificationKind, NotificationProfile, Pane,
    PaneId,
};
use crate::config::NotificationBackendName;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    pub pane_id: PaneId,
    pub pane_title: String,
    pub kind: NotificationKind,
    pub title: String,
    pub body: String,
    pub workspace_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationDecisionReason {
    WorkingBecameReady,
    EnteredNeedsAttention,
    Muted,
    ProfileFiltered,
    SelectedPane,
    CooldownActive,
}

impl NotificationDecisionReason {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkingBecameReady => "working_became_ready",
            Self::EnteredNeedsAttention => "entered_needs_attention",
            Self::Muted => "muted",
            Self::ProfileFiltered => "profile_filtered",
            Self::SelectedPane => "selected_pane",
            Self::CooldownActive => "cooldown_active",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationDecision {
    pub pane_id: PaneId,
    pub kind: NotificationKind,
    pub reason: NotificationDecisionReason,
    pub request: Option<NotificationRequest>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotificationPolicyContext<'a> {
    pub selected_pane_id: Option<&'a PaneId>,
    pub muted: bool,
    pub profile: NotificationProfile,
    pub refresh_tick: u64,
    pub cooldown_ticks: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationDispatchReceipt {
    pub backend_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationError {
    Unavailable(String),
    CommandFailed { backend: String, stderr: String },
    DispatchFailed { attempts: Vec<(String, String)> },
    Io(String),
}

impl fmt::Display for NotificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "{message}"),
            Self::CommandFailed { backend, stderr } => write!(f, "{backend}: {stderr}"),
            Self::DispatchFailed { attempts } => {
                let rendered = attempts
                    .iter()
                    .map(|(backend, error)| format!("{backend}={error}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "all notification backends failed: {rendered}")
            }
            Self::Io(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for NotificationError {}

pub trait NotificationBackend {
    fn name(&self) -> &str;
    fn send(&self, request: &NotificationRequest) -> Result<(), NotificationError>;
}

#[derive(Default)]
pub struct NotificationDispatcher {
    backends: Vec<Box<dyn NotificationBackend>>,
}

impl NotificationDispatcher {
    pub fn new(backends: Vec<Box<dyn NotificationBackend>>) -> Self {
        Self { backends }
    }

    pub fn backend_names(&self) -> Vec<String> {
        self.backends
            .iter()
            .map(|backend| backend.name().to_string())
            .collect()
    }

    pub fn dispatch(
        &self,
        request: &NotificationRequest,
    ) -> Result<NotificationDispatchReceipt, NotificationError> {
        let mut attempts = Vec::new();

        for backend in &self.backends {
            match backend.send(request) {
                Ok(()) => {
                    return Ok(NotificationDispatchReceipt {
                        backend_name: backend.name().to_string(),
                    });
                }
                Err(error) => attempts.push((backend.name().to_string(), error.to_string())),
            }
        }

        if attempts.is_empty() {
            return Err(NotificationError::Unavailable(
                "no notification backends configured".to_string(),
            ));
        }

        Err(NotificationError::DispatchFailed { attempts })
    }
}

#[derive(Debug, Clone)]
pub struct CommandNotificationBackend {
    name: String,
    program: PathBuf,
    args: Vec<String>,
}

impl CommandNotificationBackend {
    pub fn new<I, S>(name: impl Into<String>, program: impl Into<PathBuf>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            name: name.into(),
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

impl NotificationBackend for CommandNotificationBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn send(&self, request: &NotificationRequest) -> Result<(), NotificationError> {
        let output = Command::new(&self.program)
            .args(&self.args)
            .env("FOREMAN_NOTIFY_TITLE", &request.title)
            .env("FOREMAN_NOTIFY_BODY", &request.body)
            .env("FOREMAN_NOTIFY_KIND", request.kind.label())
            .env("FOREMAN_NOTIFY_PANE_ID", request.pane_id.as_str())
            .env("FOREMAN_NOTIFY_PANE_TITLE", &request.pane_title)
            .env(
                "FOREMAN_NOTIFY_WORKSPACE",
                request
                    .workspace_path
                    .as_deref()
                    .map(|path| path.as_os_str())
                    .unwrap_or_default(),
            )
            .output()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    NotificationError::Unavailable(format!(
                        "{} is not installed",
                        self.program.display()
                    ))
                } else {
                    NotificationError::Io(error.to_string())
                }
            })?;

        if output.status.success() {
            return Ok(());
        }

        Err(NotificationError::CommandFailed {
            backend: self.name.clone(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

pub fn build_notification_dispatcher(
    backends: &[NotificationBackendName],
) -> NotificationDispatcher {
    NotificationDispatcher::new(backends.iter().copied().map(configured_backend).collect())
}

pub fn evaluate_inventory_notifications(
    previous: &Inventory,
    current: &Inventory,
    context: NotificationPolicyContext<'_>,
    cooldowns: &std::collections::BTreeMap<NotificationCooldownKey, u64>,
) -> Vec<NotificationDecision> {
    current
        .sessions
        .iter()
        .flat_map(|session| session.windows.iter())
        .flat_map(|window| window.panes.iter())
        .filter_map(|pane| {
            let current_agent = pane.agent.as_ref()?;
            let previous_agent = previous.pane(&pane.id)?.agent.as_ref()?;
            let (kind, transition_reason) =
                transition_kind(previous_agent.status, current_agent.status)?;

            if context.muted {
                return Some(NotificationDecision {
                    pane_id: pane.id.clone(),
                    kind,
                    reason: NotificationDecisionReason::Muted,
                    request: None,
                });
            }

            if !context.profile.allows(kind) {
                return Some(NotificationDecision {
                    pane_id: pane.id.clone(),
                    kind,
                    reason: NotificationDecisionReason::ProfileFiltered,
                    request: None,
                });
            }

            if context.selected_pane_id == Some(&pane.id) {
                return Some(NotificationDecision {
                    pane_id: pane.id.clone(),
                    kind,
                    reason: NotificationDecisionReason::SelectedPane,
                    request: None,
                });
            }

            let cooldown_key = NotificationCooldownKey {
                pane_id: pane.id.clone(),
                kind,
            };
            if cooldowns.get(&cooldown_key).is_some_and(|last_tick| {
                context.refresh_tick.saturating_sub(*last_tick) < context.cooldown_ticks
            }) {
                return Some(NotificationDecision {
                    pane_id: pane.id.clone(),
                    kind,
                    reason: NotificationDecisionReason::CooldownActive,
                    request: None,
                });
            }

            Some(NotificationDecision {
                pane_id: pane.id.clone(),
                kind,
                reason: transition_reason,
                request: Some(notification_request(pane, kind)),
            })
        })
        .collect()
}

fn configured_backend(name: NotificationBackendName) -> Box<dyn NotificationBackend> {
    match name {
        NotificationBackendName::NotifySend => Box::new(CommandNotificationBackend::new(
            name.label(),
            "sh",
            [
                "-c",
                r#"notify-send "$FOREMAN_NOTIFY_TITLE" "$FOREMAN_NOTIFY_BODY""#,
            ],
        )),
        NotificationBackendName::OsaScript => Box::new(CommandNotificationBackend::new(
            name.label(),
            "sh",
            [
                "-c",
                r#"osascript -e 'display notification (system attribute "FOREMAN_NOTIFY_BODY") with title (system attribute "FOREMAN_NOTIFY_TITLE")'"#,
            ],
        )),
    }
}

fn transition_kind(
    previous: AgentStatus,
    current: AgentStatus,
) -> Option<(NotificationKind, NotificationDecisionReason)> {
    if previous == AgentStatus::Working && current == AgentStatus::Idle {
        return Some((
            NotificationKind::Completion,
            NotificationDecisionReason::WorkingBecameReady,
        ));
    }

    if previous != AgentStatus::NeedsAttention && current == AgentStatus::NeedsAttention {
        return Some((
            NotificationKind::NeedsAttention,
            NotificationDecisionReason::EnteredNeedsAttention,
        ));
    }

    None
}

fn notification_request(pane: &Pane, kind: NotificationKind) -> NotificationRequest {
    let (title, body) = match kind {
        NotificationKind::Completion => (
            format!("Agent ready: {}", pane.title),
            "The agent returned to an idle state.".to_string(),
        ),
        NotificationKind::NeedsAttention => (
            format!("Needs attention: {}", pane.title),
            "The agent is waiting for input or intervention.".to_string(),
        ),
    };

    NotificationRequest {
        pane_id: pane.id.clone(),
        pane_title: pane.title.clone(),
        kind,
        title,
        body,
        workspace_path: pane.working_dir.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_notification_dispatcher, evaluate_inventory_notifications, NotificationBackend,
        NotificationDecisionReason, NotificationDispatcher, NotificationError,
        NotificationPolicyContext,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, NotificationCooldownKey, NotificationKind,
        NotificationProfile, PaneBuilder, SessionBuilder, WindowBuilder,
    };
    use crate::config::NotificationBackendName;
    use std::cell::RefCell;
    use std::path::PathBuf;
    use std::rc::Rc;

    fn inventory_with_status(status: AgentStatus) -> crate::app::Inventory {
        inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .title("claude-main")
                    .status(status),
            ),
        )])
    }

    #[test]
    fn completion_transition_emits_notification_when_unsuppressed() {
        let previous = inventory_with_status(AgentStatus::Working);
        let current = inventory_with_status(AgentStatus::Idle);

        let decisions = evaluate_inventory_notifications(
            &previous,
            &current,
            NotificationPolicyContext {
                selected_pane_id: None,
                muted: false,
                profile: NotificationProfile::All,
                refresh_tick: 4,
                cooldown_ticks: 3,
            },
            &Default::default(),
        );

        assert_eq!(decisions.len(), 1);
        assert_eq!(
            decisions[0].reason,
            NotificationDecisionReason::WorkingBecameReady
        );
        assert_eq!(decisions[0].kind, NotificationKind::Completion);
        assert!(decisions[0].request.is_some());
    }

    #[test]
    fn selected_pane_suppresses_attention_notification() {
        let previous = inventory_with_status(AgentStatus::Working);
        let current = inventory_with_status(AgentStatus::NeedsAttention);

        let decisions = evaluate_inventory_notifications(
            &previous,
            &current,
            NotificationPolicyContext {
                selected_pane_id: Some(&"alpha:claude".into()),
                muted: false,
                profile: NotificationProfile::All,
                refresh_tick: 4,
                cooldown_ticks: 3,
            },
            &Default::default(),
        );

        assert_eq!(decisions.len(), 1);
        assert_eq!(
            decisions[0].reason,
            NotificationDecisionReason::SelectedPane
        );
        assert!(decisions[0].request.is_none());
    }

    #[test]
    fn profile_and_cooldown_can_suppress_transitions() {
        let previous = inventory_with_status(AgentStatus::Working);
        let current = inventory_with_status(AgentStatus::Idle);
        let cooldowns = std::collections::BTreeMap::from([(
            NotificationCooldownKey {
                pane_id: "alpha:claude".into(),
                kind: NotificationKind::Completion,
            },
            3,
        )]);

        let profile_filtered = evaluate_inventory_notifications(
            &previous,
            &current,
            NotificationPolicyContext {
                selected_pane_id: None,
                muted: false,
                profile: NotificationProfile::AttentionOnly,
                refresh_tick: 5,
                cooldown_ticks: 3,
            },
            &Default::default(),
        );
        assert_eq!(
            profile_filtered[0].reason,
            NotificationDecisionReason::ProfileFiltered
        );

        let cooldown_filtered = evaluate_inventory_notifications(
            &previous,
            &current,
            NotificationPolicyContext {
                selected_pane_id: None,
                muted: false,
                profile: NotificationProfile::All,
                refresh_tick: 5,
                cooldown_ticks: 3,
            },
            &cooldowns,
        );
        assert_eq!(
            cooldown_filtered[0].reason,
            NotificationDecisionReason::CooldownActive
        );

        let custom_cooldown = evaluate_inventory_notifications(
            &previous,
            &current,
            NotificationPolicyContext {
                selected_pane_id: None,
                muted: false,
                profile: NotificationProfile::All,
                refresh_tick: 5,
                cooldown_ticks: 2,
            },
            &cooldowns,
        );
        assert_eq!(
            custom_cooldown[0].reason,
            NotificationDecisionReason::WorkingBecameReady
        );
    }

    #[derive(Clone)]
    struct FakeBackend {
        name: String,
        should_fail: bool,
        calls: Rc<RefCell<Vec<String>>>,
    }

    impl FakeBackend {
        fn new(name: &str, should_fail: bool, calls: Rc<RefCell<Vec<String>>>) -> Self {
            Self {
                name: name.to_string(),
                should_fail,
                calls,
            }
        }
    }

    impl NotificationBackend for FakeBackend {
        fn name(&self) -> &str {
            &self.name
        }

        fn send(&self, request: &super::NotificationRequest) -> Result<(), NotificationError> {
            self.calls
                .borrow_mut()
                .push(format!("{}:{}", self.name, request.kind.label()));
            if self.should_fail {
                Err(NotificationError::Unavailable(format!(
                    "{} unavailable",
                    self.name
                )))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn dispatcher_falls_back_to_later_backend() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let dispatcher = NotificationDispatcher::new(vec![
            Box::new(FakeBackend::new("primary", true, Rc::clone(&calls))),
            Box::new(FakeBackend::new("fallback", false, Rc::clone(&calls))),
        ]);
        let request = super::NotificationRequest {
            pane_id: "alpha:claude".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::Completion,
            title: "Agent ready: claude-main".to_string(),
            body: "The agent returned to an idle state.".to_string(),
            workspace_path: Some(PathBuf::from("/tmp/alpha")),
        };

        let receipt = dispatcher
            .dispatch(&request)
            .expect("fallback backend should succeed");

        assert_eq!(receipt.backend_name, "fallback");
        assert_eq!(
            calls.borrow().as_slice(),
            &[
                "primary:completion".to_string(),
                "fallback:completion".to_string()
            ]
        );
    }

    #[test]
    fn dispatcher_reports_aggregate_failure_when_all_backends_fail() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let dispatcher = NotificationDispatcher::new(vec![Box::new(FakeBackend::new(
            "primary",
            true,
            Rc::clone(&calls),
        ))]);
        let request = super::NotificationRequest {
            pane_id: "alpha:claude".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::NeedsAttention,
            title: "Needs attention: claude-main".to_string(),
            body: "The agent is waiting for input or intervention.".to_string(),
            workspace_path: Some(PathBuf::from("/tmp/alpha")),
        };

        let error = dispatcher
            .dispatch(&request)
            .expect_err("all backends should fail");

        assert!(matches!(error, NotificationError::DispatchFailed { .. }));
        assert_eq!(
            calls.borrow().as_slice(),
            &["primary:needs_attention".to_string()]
        );
    }

    #[test]
    fn configured_dispatcher_keeps_requested_backend_order() {
        let dispatcher = build_notification_dispatcher(&[
            NotificationBackendName::OsaScript,
            NotificationBackendName::NotifySend,
        ]);
        assert_eq!(
            dispatcher.backend_names(),
            vec!["osascript".to_string(), "notify-send".to_string()]
        );
    }
}
