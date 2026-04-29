use crate::app::{
    AgentStatus, Inventory, NotificationCooldownKey, NotificationKind, NotificationProfile, Pane,
    PaneId,
};
use crate::config::{NotificationBackendName, NotificationSoundCycle, NotificationSoundProfile};
use std::collections::BTreeMap;
use std::fmt;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    pub pane_id: PaneId,
    pub pane_title: String,
    pub kind: NotificationKind,
    pub title: String,
    pub subtitle: String,
    pub body: String,
    pub audible: bool,
    pub window_target: Option<String>,
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
    fn send(&self, delivery: &NotificationDelivery<'_>) -> Result<(), NotificationError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationDelivery<'a> {
    pub request: &'a NotificationRequest,
    pub sound: ResolvedNotificationSound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedNotificationSound {
    System(String),
    File(PathBuf),
    None,
}

pub struct NotificationDispatcher {
    backends: Vec<Box<dyn NotificationBackend>>,
    sounds: NotificationSoundSelector,
}

impl NotificationDispatcher {
    pub fn new(backends: Vec<Box<dyn NotificationBackend>>) -> Self {
        Self {
            backends,
            sounds: NotificationSoundSelector::default(),
        }
    }

    pub fn with_sounds(
        backends: Vec<Box<dyn NotificationBackend>>,
        sounds: NotificationSoundSelector,
    ) -> Self {
        Self { backends, sounds }
    }

    pub fn backend_names(&self) -> Vec<String> {
        self.backends
            .iter()
            .map(|backend| backend.name().to_string())
            .collect()
    }

    pub fn dispatch(
        &mut self,
        request: &NotificationRequest,
    ) -> Result<NotificationDispatchReceipt, NotificationError> {
        let mut attempts = Vec::new();
        let sound = if request.audible {
            self.sounds.resolve(request.kind)
        } else {
            ResolvedNotificationSound::None
        };

        for backend in &self.backends {
            let delivery = NotificationDelivery {
                request,
                sound: sound.clone(),
            };
            match backend.send(&delivery) {
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

impl Default for NotificationDispatcher {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationSoundSelector {
    completion: NotificationSoundSource,
    needs_attention: NotificationSoundSource,
}

impl NotificationSoundSelector {
    pub fn from_profiles(
        active_profile: &str,
        profiles: &BTreeMap<String, NotificationSoundProfile>,
        config_dir: Option<&Path>,
    ) -> Self {
        let profile = profiles
            .get(active_profile)
            .or_else(|| profiles.get("default"))
            .cloned()
            .unwrap_or_default();

        Self {
            completion: NotificationSoundSource::from_config(
                &profile.completion,
                profile.cycle,
                config_dir,
            ),
            needs_attention: NotificationSoundSource::from_config(
                &profile.needs_attention,
                profile.cycle,
                config_dir,
            ),
        }
    }

    fn resolve(&mut self, kind: NotificationKind) -> ResolvedNotificationSound {
        match kind {
            NotificationKind::Completion => self.completion.resolve(),
            NotificationKind::NeedsAttention => self.needs_attention.resolve(),
        }
    }
}

impl Default for NotificationSoundSelector {
    fn default() -> Self {
        let profile = NotificationSoundProfile::default();
        Self {
            completion: NotificationSoundSource::from_config(
                &profile.completion,
                profile.cycle,
                None,
            ),
            needs_attention: NotificationSoundSource::from_config(
                &profile.needs_attention,
                profile.cycle,
                None,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NotificationSoundSource {
    System(String),
    Directory {
        files: Vec<PathBuf>,
        cycle: NotificationSoundCycle,
        index: usize,
    },
    File(PathBuf),
    None,
}

impl NotificationSoundSource {
    fn from_config(value: &str, cycle: NotificationSoundCycle, config_dir: Option<&Path>) -> Self {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            return Self::None;
        }

        let path = expand_sound_path(trimmed, config_dir);
        if path.is_dir() {
            let mut files = std::fs::read_dir(&path)
                .ok()
                .into_iter()
                .flat_map(|entries| entries.filter_map(Result::ok))
                .map(|entry| entry.path())
                .filter(|candidate| {
                    candidate.is_file()
                        && candidate
                            .extension()
                            .and_then(|extension| extension.to_str())
                            .is_some_and(is_audio_extension)
                })
                .collect::<Vec<_>>();
            files.sort();

            if files.is_empty() {
                return Self::None;
            }

            return Self::Directory {
                files,
                cycle,
                index: 0,
            };
        }

        if path.is_file() {
            return Self::File(path);
        }

        if looks_like_path(trimmed) {
            return Self::None;
        }

        Self::System(trimmed.to_string())
    }

    fn resolve(&mut self) -> ResolvedNotificationSound {
        match self {
            Self::System(name) => ResolvedNotificationSound::System(name.clone()),
            Self::File(path) => ResolvedNotificationSound::File(path.clone()),
            Self::Directory {
                files,
                cycle,
                index,
            } => {
                if files.is_empty() {
                    return ResolvedNotificationSound::None;
                }
                let selected = match cycle {
                    NotificationSoundCycle::Sequential => {
                        let selected = *index % files.len();
                        *index = index.saturating_add(1);
                        selected
                    }
                    NotificationSoundCycle::Random => pseudo_random_index(files.len(), *index),
                };
                ResolvedNotificationSound::File(files[selected].clone())
            }
            Self::None => ResolvedNotificationSound::None,
        }
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

    fn send(&self, delivery: &NotificationDelivery<'_>) -> Result<(), NotificationError> {
        let request = delivery.request;
        let output = Command::new(&self.program)
            .args(&self.args)
            .env("FOREMAN_NOTIFY_TITLE", &request.title)
            .env("FOREMAN_NOTIFY_SUBTITLE", &request.subtitle)
            .env("FOREMAN_NOTIFY_BODY", &request.body)
            .env("FOREMAN_NOTIFY_KIND", request.kind.label())
            .env("FOREMAN_NOTIFY_PANE_ID", request.pane_id.as_str())
            .env("FOREMAN_NOTIFY_PANE_TITLE", &request.pane_title)
            .env(
                "FOREMAN_NOTIFY_WINDOW_TARGET",
                request.window_target.as_deref().unwrap_or_default(),
            )
            .env("FOREMAN_NOTIFY_SOUND", sound_label(&delivery.sound))
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
    sound_profile: &str,
    sound_profiles: &BTreeMap<String, NotificationSoundProfile>,
    config_dir: Option<&Path>,
    tmux_socket: Option<PathBuf>,
    diagnostic_log_path: Option<PathBuf>,
) -> NotificationDispatcher {
    NotificationDispatcher::with_sounds(
        backends
            .iter()
            .copied()
            .map(|backend| {
                configured_backend(backend, tmux_socket.clone(), diagnostic_log_path.clone())
            })
            .collect(),
        NotificationSoundSelector::from_profiles(sound_profile, sound_profiles, config_dir),
    )
}

pub fn evaluate_inventory_notifications(
    previous: &Inventory,
    current: &Inventory,
    context: NotificationPolicyContext<'_>,
    cooldowns: &std::collections::BTreeMap<NotificationCooldownKey, u64>,
) -> Vec<NotificationDecision> {
    let mut decisions = Vec::new();
    for session in &current.sessions {
        for window in &session.windows {
            for pane in &window.panes {
                let Some(current_agent) = pane.agent.as_ref() else {
                    continue;
                };
                let Some(previous_agent) =
                    previous.pane(&pane.id).and_then(|pane| pane.agent.as_ref())
                else {
                    continue;
                };
                let Some((kind, transition_reason)) =
                    transition_kind(previous_agent.status, current_agent.status)
                else {
                    continue;
                };

                if context.muted {
                    decisions.push(NotificationDecision {
                        pane_id: pane.id.clone(),
                        kind,
                        reason: NotificationDecisionReason::Muted,
                        request: None,
                    });
                    continue;
                }

                if !context.profile.allows(kind) {
                    decisions.push(NotificationDecision {
                        pane_id: pane.id.clone(),
                        kind,
                        reason: NotificationDecisionReason::ProfileFiltered,
                        request: None,
                    });
                    continue;
                }

                if context.selected_pane_id == Some(&pane.id) {
                    decisions.push(NotificationDecision {
                        pane_id: pane.id.clone(),
                        kind,
                        reason: NotificationDecisionReason::SelectedPane,
                        request: None,
                    });
                    continue;
                }

                let cooldown_key = NotificationCooldownKey {
                    pane_id: pane.id.clone(),
                    kind,
                };
                if cooldowns.get(&cooldown_key).is_some_and(|last_tick| {
                    context.refresh_tick.saturating_sub(*last_tick) < context.cooldown_ticks
                }) {
                    decisions.push(NotificationDecision {
                        pane_id: pane.id.clone(),
                        kind,
                        reason: NotificationDecisionReason::CooldownActive,
                        request: None,
                    });
                    continue;
                }

                decisions.push(NotificationDecision {
                    pane_id: pane.id.clone(),
                    kind,
                    reason: transition_reason,
                    request: Some(notification_request(
                        pane,
                        Some(NotificationLocationContext {
                            session_name: &session.name,
                            window_id: window.id.as_str(),
                            window_name: &window.name,
                        }),
                        kind,
                    )),
                });
            }
        }
    }

    decisions
}

pub fn coalesce_notification_requests(
    requests: Vec<NotificationRequest>,
) -> Vec<NotificationRequest> {
    let mut groups: Vec<(NotificationKind, Vec<NotificationRequest>)> = Vec::new();

    for request in requests {
        if let Some((_, existing)) = groups.iter_mut().find(|(kind, _)| *kind == request.kind) {
            existing.push(request);
        } else {
            groups.push((request.kind, vec![request]));
        }
    }

    groups
        .into_iter()
        .map(|(kind, requests)| grouped_notification_request(kind, requests))
        .collect()
}

#[derive(Debug, Clone)]
pub struct AlerterNotificationBackend {
    alerter_program: PathBuf,
    tmux_program: PathBuf,
    afplay_program: PathBuf,
    tmux_socket: Option<PathBuf>,
    diagnostic_log_path: Option<PathBuf>,
}

impl AlerterNotificationBackend {
    pub fn new(tmux_socket: Option<PathBuf>, diagnostic_log_path: Option<PathBuf>) -> Self {
        Self {
            alerter_program: "alerter".into(),
            tmux_program: "tmux".into(),
            afplay_program: "afplay".into(),
            tmux_socket,
            diagnostic_log_path,
        }
    }

    #[cfg(test)]
    fn with_programs(
        alerter_program: impl Into<PathBuf>,
        tmux_program: impl Into<PathBuf>,
        afplay_program: impl Into<PathBuf>,
        tmux_socket: Option<PathBuf>,
        diagnostic_log_path: Option<PathBuf>,
    ) -> Self {
        Self {
            alerter_program: alerter_program.into(),
            tmux_program: tmux_program.into(),
            afplay_program: afplay_program.into(),
            tmux_socket,
            diagnostic_log_path,
        }
    }
}

impl NotificationBackend for AlerterNotificationBackend {
    fn name(&self) -> &str {
        "alerter"
    }

    fn send(&self, delivery: &NotificationDelivery<'_>) -> Result<(), NotificationError> {
        if !command_available(&self.alerter_program) {
            return Err(NotificationError::Unavailable(format!(
                "{} is not installed",
                self.alerter_program.display()
            )));
        }

        let request = delivery.request.clone();
        let sound = delivery.sound.clone();
        let alerter_program = self.alerter_program.clone();
        let tmux_program = self.tmux_program.clone();
        let afplay_program = self.afplay_program.clone();
        let tmux_socket = self.tmux_socket.clone();
        let diagnostic_log_path = self.diagnostic_log_path.clone();

        write_notification_diagnostic(
            diagnostic_log_path.as_deref(),
            "INFO",
            &format!(
                "notification_alerter_started pane_id={} title={} subtitle={} sound={} tmux_socket={} tmux_env={}",
                request.pane_id.as_str(),
                log_field(&request.title),
                log_field(&request.subtitle),
                log_field(&sound_label(&sound)),
                tmux_socket
                    .as_deref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "default".to_string()),
                std::env::var_os("TMUX").is_some()
            ),
        );

        let mut command = Command::new(&alerter_program);
        command
            .arg("--title")
            .arg(&request.title)
            .arg("--subtitle")
            .arg(&request.subtitle)
            .arg("--message")
            .arg(&request.body)
            .arg("--group")
            .arg(format!("foreman:{}", request.pane_id.as_str()))
            .arg("--timeout")
            .arg("10")
            .arg("--actions")
            .arg("Open tmux pane")
            .arg("--close-label")
            .arg("Dismiss")
            .arg("--json")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let ResolvedNotificationSound::System(sound_name) = &sound {
            command.arg("--sound").arg(sound_name);
        }

        let mut child = spawn_alerter_command(&mut command).map_err(|error| {
            write_notification_diagnostic(
                diagnostic_log_path.as_deref(),
                "WARN",
                &format!(
                    "notification_alerter_failed_to_spawn pane_id={} program={} error={}",
                    request.pane_id.as_str(),
                    alerter_program.display(),
                    log_field(&error.to_string())
                ),
            );
            if error.kind() == std::io::ErrorKind::NotFound {
                NotificationError::Unavailable(format!(
                    "{} is not installed",
                    alerter_program.display()
                ))
            } else {
                NotificationError::Io(error.to_string())
            }
        })?;

        let mut exited_early = false;
        for _ in 0..20 {
            if child
                .try_wait()
                .map_err(|error| NotificationError::Io(error.to_string()))?
                .is_some()
            {
                exited_early = true;
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        if exited_early {
            let output = child
                .wait_with_output()
                .map_err(|error| NotificationError::Io(error.to_string()))?;
            if output.status.success() {
                log_alerter_sound_result(
                    &request,
                    &sound,
                    &afplay_program,
                    diagnostic_log_path.as_deref(),
                );
            }
            let succeeded = handle_alerter_output(
                &request,
                &tmux_program,
                tmux_socket.as_deref(),
                diagnostic_log_path.as_deref(),
                output,
            );
            return if succeeded {
                Ok(())
            } else {
                Err(NotificationError::CommandFailed {
                    backend: self.name().to_string(),
                    stderr: "alerter exited before delivery completed".to_string(),
                })
            };
        }

        log_alerter_sound_result(
            &request,
            &sound,
            &afplay_program,
            diagnostic_log_path.as_deref(),
        );

        thread::spawn(move || {
            let output = match child.wait_with_output() {
                Ok(output) => output,
                Err(error) => {
                    write_notification_diagnostic(
                        diagnostic_log_path.as_deref(),
                        "WARN",
                        &format!(
                            "notification_alerter_wait_failed pane_id={} error={}",
                            request.pane_id.as_str(),
                            log_field(&error.to_string())
                        ),
                    );
                    return;
                }
            };
            handle_alerter_output(
                &request,
                &tmux_program,
                tmux_socket.as_deref(),
                diagnostic_log_path.as_deref(),
                output,
            );
        });

        Ok(())
    }
}

fn spawn_alerter_command(command: &mut Command) -> io::Result<Child> {
    const TEXT_FILE_BUSY: i32 = 26;

    for _ in 0..5 {
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error) if error.raw_os_error() == Some(TEXT_FILE_BUSY) => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => return Err(error),
        }
    }

    command.spawn()
}

#[derive(Debug, Clone)]
struct OsaScriptNotificationBackend {
    program: PathBuf,
    afplay_program: PathBuf,
}

impl OsaScriptNotificationBackend {
    fn new() -> Self {
        Self {
            program: "osascript".into(),
            afplay_program: "afplay".into(),
        }
    }
}

impl NotificationBackend for OsaScriptNotificationBackend {
    fn name(&self) -> &str {
        "osascript"
    }

    fn send(&self, delivery: &NotificationDelivery<'_>) -> Result<(), NotificationError> {
        let mut script = format!(
            "display notification {} with title {} subtitle {}",
            apple_script_string(&delivery.request.body),
            apple_script_string(&delivery.request.title),
            apple_script_string(&delivery.request.subtitle)
        );
        if let ResolvedNotificationSound::System(sound_name) = &delivery.sound {
            script.push_str(" sound name ");
            script.push_str(&apple_script_string(sound_name));
        }

        let output = Command::new(&self.program)
            .arg("-e")
            .arg(script)
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
            let _ = play_file_sound_if_needed(&delivery.sound, &self.afplay_program);
            return Ok(());
        }

        Err(NotificationError::CommandFailed {
            backend: self.name().to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

fn configured_backend(
    name: NotificationBackendName,
    tmux_socket: Option<PathBuf>,
    diagnostic_log_path: Option<PathBuf>,
) -> Box<dyn NotificationBackend> {
    match name {
        NotificationBackendName::Alerter => Box::new(AlerterNotificationBackend::new(
            tmux_socket,
            diagnostic_log_path,
        )),
        NotificationBackendName::NotifySend => Box::new(CommandNotificationBackend::new(
            name.label(),
            "sh",
            [
                "-c",
                r#"notify-send "$FOREMAN_NOTIFY_TITLE" "$FOREMAN_NOTIFY_SUBTITLE
$FOREMAN_NOTIFY_BODY""#,
            ],
        )),
        NotificationBackendName::OsaScript => Box::new(OsaScriptNotificationBackend::new()),
    }
}

fn sound_label(sound: &ResolvedNotificationSound) -> String {
    match sound {
        ResolvedNotificationSound::System(name) => name.clone(),
        ResolvedNotificationSound::File(path) => path.display().to_string(),
        ResolvedNotificationSound::None => String::new(),
    }
}

fn log_field(value: &str) -> String {
    if value.is_empty() {
        return "-".to_string();
    }
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace(' ', "\\s")
}

fn write_notification_diagnostic(log_path: Option<&Path>, level: &str, message: &str) {
    let Some(log_path) = log_path else {
        return;
    };
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) else {
        return;
    };
    let _ = writeln!(file, "[{level}] {message}");
}

fn log_alerter_sound_result(
    request: &NotificationRequest,
    sound: &ResolvedNotificationSound,
    afplay_program: &Path,
    diagnostic_log_path: Option<&Path>,
) {
    match play_file_sound_if_needed(sound, afplay_program) {
        Ok(true) => write_notification_diagnostic(
            diagnostic_log_path,
            "INFO",
            &format!(
                "notification_alerter_sound_spawned pane_id={} sound={}",
                request.pane_id.as_str(),
                log_field(&sound_label(sound))
            ),
        ),
        Ok(false) => {}
        Err(error) => write_notification_diagnostic(
            diagnostic_log_path,
            "WARN",
            &format!(
                "notification_alerter_sound_failed pane_id={} error={}",
                request.pane_id.as_str(),
                log_field(&error)
            ),
        ),
    }
}

fn handle_alerter_output(
    request: &NotificationRequest,
    tmux_program: &Path,
    tmux_socket: Option<&Path>,
    diagnostic_log_path: Option<&Path>,
    output: Output,
) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    write_notification_diagnostic(
        diagnostic_log_path,
        "INFO",
        &format!(
            "notification_alerter_completed pane_id={} status={} stdout={} stderr={}",
            request.pane_id.as_str(),
            output.status,
            log_field(&stdout),
            log_field(&stderr)
        ),
    );
    if !output.status.success() {
        return false;
    }

    if alerter_output_requests_focus(&stdout) {
        write_notification_diagnostic(
            diagnostic_log_path,
            "INFO",
            &format!(
                "notification_alerter_focus_requested pane_id={} output={}",
                request.pane_id.as_str(),
                log_field(&stdout)
            ),
        );
        select_tmux_target(
            tmux_program,
            tmux_socket,
            request.window_target.as_deref(),
            request.pane_id.as_str(),
            diagnostic_log_path,
        );
    } else {
        write_notification_diagnostic(
            diagnostic_log_path,
            "INFO",
            &format!(
                "notification_alerter_no_focus pane_id={} output={}",
                request.pane_id.as_str(),
                log_field(&stdout)
            ),
        );
    }
    true
}

fn alerter_output_requests_focus(output: &str) -> bool {
    let trimmed = output.trim();
    matches!(
        trimmed,
        "Open tmux pane" | "Go to pane" | "@ACTIONCLICKED" | "@CONTENTCLICKED"
    ) || trimmed.contains(r#""activationType" : "contentsClicked""#)
        || trimmed.contains(r#""activationType":"contentsClicked""#)
        || trimmed.contains(r#""activationType" : "actionClicked""#)
        || trimmed.contains(r#""activationType":"actionClicked""#)
        || trimmed.contains(r#""activationValue" : "Open tmux pane""#)
        || trimmed.contains(r#""activationValue":"Open tmux pane""#)
        || trimmed.contains(r#""activationValue" : "Go to pane""#)
        || trimmed.contains(r#""activationValue":"Go to pane""#)
}

fn expand_sound_path(value: &str, config_dir: Option<&Path>) -> PathBuf {
    if let Some(stripped) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }

    let path = PathBuf::from(value);
    if path.is_absolute() {
        return path;
    }

    config_dir.map(|dir| dir.join(&path)).unwrap_or(path)
}

fn looks_like_path(value: &str) -> bool {
    value.contains('/') || value.contains('\\') || value.starts_with('.')
}

fn is_audio_extension(extension: &str) -> bool {
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "aiff" | "aif" | "caf" | "wav" | "mp3"
    )
}

fn pseudo_random_index(len: usize, salt: usize) -> usize {
    if len == 0 {
        return 0;
    }

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or(0);
    nanos.wrapping_add(salt) % len
}

fn command_available(program: &Path) -> bool {
    if program.components().count() > 1 || program.is_absolute() {
        return program.is_file();
    }

    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|dir| dir.join(program).is_file())
}

fn play_file_sound_if_needed(
    sound: &ResolvedNotificationSound,
    afplay_program: &Path,
) -> Result<bool, String> {
    let ResolvedNotificationSound::File(path) = sound else {
        return Ok(false);
    };

    let child = Command::new(afplay_program)
        .arg(path)
        .spawn()
        .map_err(|error| error.to_string())?;

    #[cfg(test)]
    {
        let mut child = child;
        child.wait().map_err(|error| error.to_string())?;
    }
    #[cfg(not(test))]
    {
        let _ = child;
    }

    Ok(true)
}

fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn select_tmux_target(
    tmux_program: &Path,
    tmux_socket: Option<&Path>,
    window_target: Option<&str>,
    pane_target: &str,
    diagnostic_log_path: Option<&Path>,
) {
    let live_target = resolve_tmux_focus_target(tmux_program, tmux_socket, pane_target);
    if let Some((session_id, window_id)) = live_target {
        write_notification_diagnostic(
            diagnostic_log_path,
            "INFO",
            &format!(
                "notification_tmux_resolved pane_id={} session_id={} window_id={}",
                pane_target,
                log_field(&session_id),
                log_field(&window_id)
            ),
        );
        run_tmux_diagnostic_command(
            tmux_program,
            tmux_socket,
            &["switch-client", "-t", &session_id],
            diagnostic_log_path,
            pane_target,
        );

        run_tmux_diagnostic_command(
            tmux_program,
            tmux_socket,
            &["select-window", "-t", &window_id],
            diagnostic_log_path,
            pane_target,
        );
    } else if let Some(window_target) = window_target.filter(|target| !target.is_empty()) {
        write_notification_diagnostic(
            diagnostic_log_path,
            "WARN",
            &format!(
                "notification_tmux_resolve_failed pane_id={} fallback_window_target={}",
                pane_target,
                log_field(window_target)
            ),
        );
        run_tmux_diagnostic_command(
            tmux_program,
            tmux_socket,
            &["select-window", "-t", window_target],
            diagnostic_log_path,
            pane_target,
        );
    }

    run_tmux_diagnostic_command(
        tmux_program,
        tmux_socket,
        &["select-pane", "-t", pane_target],
        diagnostic_log_path,
        pane_target,
    );
}

fn resolve_tmux_focus_target(
    tmux_program: &Path,
    tmux_socket: Option<&Path>,
    pane_target: &str,
) -> Option<(String, String)> {
    let mut command = tmux_command(tmux_program, tmux_socket);
    let output = command
        .arg("display-message")
        .arg("-p")
        .arg("-t")
        .arg(pane_target)
        .arg("#{session_id}\t#{window_id}")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let target = String::from_utf8_lossy(&output.stdout);
    let (session_id, window_id) = target.trim().split_once('\t')?;
    Some((session_id.to_string(), window_id.to_string()))
}

fn run_tmux_diagnostic_command(
    tmux_program: &Path,
    tmux_socket: Option<&Path>,
    args: &[&str],
    diagnostic_log_path: Option<&Path>,
    pane_target: &str,
) {
    let mut command = tmux_command(tmux_program, tmux_socket);
    let output = command.args(args).output();
    match output {
        Ok(output) => {
            write_notification_diagnostic(
                diagnostic_log_path,
                if output.status.success() {
                    "INFO"
                } else {
                    "WARN"
                },
                &format!(
                    "notification_tmux_command pane_id={} command={} status={} stdout={} stderr={}",
                    pane_target,
                    log_field(&args.join(" ")),
                    output.status,
                    log_field(String::from_utf8_lossy(&output.stdout).trim()),
                    log_field(String::from_utf8_lossy(&output.stderr).trim())
                ),
            );
        }
        Err(error) => {
            write_notification_diagnostic(
                diagnostic_log_path,
                "WARN",
                &format!(
                    "notification_tmux_command_failed pane_id={} command={} error={}",
                    pane_target,
                    log_field(&args.join(" ")),
                    log_field(&error.to_string())
                ),
            );
        }
    }
}

fn tmux_command(program: &Path, socket: Option<&Path>) -> Command {
    let mut command = Command::new(program);
    command.stderr(Stdio::null());
    if let Some(socket) = socket {
        command.arg("-S").arg(socket);
    }
    command
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

struct NotificationLocationContext<'a> {
    session_name: &'a str,
    window_id: &'a str,
    window_name: &'a str,
}

fn notification_request(
    pane: &Pane,
    location_context: Option<NotificationLocationContext<'_>>,
    kind: NotificationKind,
) -> NotificationRequest {
    let target = pane.navigation_title();
    let (location, window_target) = location_context
        .map(|context| {
            let window_label = if context.window_name.is_empty() {
                context.window_id.to_string()
            } else {
                format!("{} \"{}\"", context.window_id, context.window_name)
            };
            (
                format!("{} / {}", context.session_name, window_label),
                Some(context.window_id.to_string()),
            )
        })
        .unwrap_or_else(|| ("tmux location unavailable".to_string(), None));
    let (title, subtitle, detail) = match kind {
        NotificationKind::Completion => (
            "Foreman: agent ready".to_string(),
            target,
            "Returned to idle".to_string(),
        ),
        NotificationKind::NeedsAttention => (
            "Foreman: needs attention".to_string(),
            target,
            "Waiting for input".to_string(),
        ),
    };

    NotificationRequest {
        pane_id: pane.id.clone(),
        pane_title: pane.title.clone(),
        kind,
        title,
        subtitle,
        body: format!("{location} - {detail}"),
        audible: true,
        window_target,
        workspace_path: pane.working_dir.clone(),
    }
}

fn grouped_notification_request(
    kind: NotificationKind,
    mut requests: Vec<NotificationRequest>,
) -> NotificationRequest {
    if requests.len() == 1 {
        return requests.remove(0);
    }

    const BODY_LINE_LIMIT: usize = 5;

    let first = requests
        .first()
        .expect("grouped request should have at least one item")
        .clone();
    let total = requests.len();
    let title = match kind {
        NotificationKind::Completion => format!("Foreman: {total} agents ready"),
        NotificationKind::NeedsAttention => format!("Foreman: {total} need attention"),
    };
    let subtitle = "Multiple agents".to_string();
    let mut body_lines = requests
        .iter()
        .take(BODY_LINE_LIMIT)
        .map(|request| format!("{} - {}", request.subtitle, request.body))
        .collect::<Vec<_>>();
    if total > BODY_LINE_LIMIT {
        body_lines.push(format!("and {} more", total - BODY_LINE_LIMIT));
    }

    NotificationRequest {
        pane_id: first.pane_id,
        pane_title: first.pane_title,
        kind,
        title,
        subtitle,
        body: body_lines.join("\n"),
        audible: true,
        window_target: first.window_target,
        workspace_path: first.workspace_path,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_notification_dispatcher, coalesce_notification_requests,
        evaluate_inventory_notifications, NotificationBackend, NotificationDecisionReason,
        NotificationDispatcher, NotificationError, NotificationPolicyContext,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, NotificationCooldownKey, NotificationKind,
        NotificationProfile, PaneBuilder, SessionBuilder, WindowBuilder,
    };
    use crate::config::NotificationBackendName;
    use crate::config::{NotificationSoundCycle, NotificationSoundProfile};
    use std::cell::RefCell;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::Command;
    use std::rc::Rc;
    use std::thread;
    use std::time::Duration;
    use tempfile::tempdir;

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

    fn write_executable(path: &std::path::Path, contents: &str) {
        fs::write(path, contents).expect("script should be written");
        let mut permissions = fs::metadata(path)
            .expect("script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("script permissions should update");
    }

    fn wait_for_file_contains(path: &std::path::Path, needle: &str) {
        for _ in 0..100 {
            if let Ok(contents) = fs::read_to_string(path) {
                if contents.contains(needle) {
                    return;
                }
            }
            thread::sleep(Duration::from_millis(20));
        }

        panic!("file {} never contained {}", path.display(), needle);
    }

    fn run_tmux(socket: &std::path::Path, args: &[&str]) -> String {
        let output = Command::new("tmux")
            .arg("-f")
            .arg("/dev/null")
            .arg("-S")
            .arg(socket)
            .args(args)
            .output()
            .expect("tmux command should run");
        assert!(
            output.status.success(),
            "tmux {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string()
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
        let request = decisions[0]
            .request
            .as_ref()
            .expect("request should be built");
        assert_eq!(request.title, "Foreman: agent ready");
        assert_eq!(request.subtitle, "alpha");
        assert!(request.body.contains("alpha / alpha:agents"));
        assert!(!request.body.contains("/tmp/alpha"));
        assert_eq!(request.window_target.as_deref(), Some("alpha:agents"));
    }

    #[test]
    fn coalesce_notification_requests_groups_same_kind_bursts() {
        let previous = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents")
                .pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .working_dir("/tmp/alpha")
                        .title("claude-main")
                        .status(AgentStatus::Working),
                )
                .pane(
                    PaneBuilder::agent("alpha:codex", HarnessKind::CodexCli)
                        .working_dir("/tmp/beta")
                        .title("codex-main")
                        .status(AgentStatus::Working),
                ),
        )]);
        let current = inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents")
                .pane(
                    PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                        .working_dir("/tmp/alpha")
                        .title("claude-main")
                        .status(AgentStatus::Idle),
                )
                .pane(
                    PaneBuilder::agent("alpha:codex", HarnessKind::CodexCli)
                        .working_dir("/tmp/beta")
                        .title("codex-main")
                        .status(AgentStatus::Idle),
                ),
        )]);

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
        let requests = decisions
            .into_iter()
            .filter_map(|decision| decision.request)
            .collect::<Vec<_>>();

        let coalesced = coalesce_notification_requests(requests);

        assert_eq!(coalesced.len(), 1);
        assert_eq!(coalesced[0].title, "Foreman: 2 agents ready");
        assert_eq!(coalesced[0].subtitle, "Multiple agents");
        assert_eq!(coalesced[0].pane_id.as_str(), "alpha:claude");
        assert_eq!(coalesced[0].window_target.as_deref(), Some("alpha:agents"));
        assert!(coalesced[0].audible);
        assert!(coalesced[0].body.contains("alpha"));
        assert!(coalesced[0].body.contains("beta"));
        assert!(!coalesced[0].body.contains("/tmp/alpha"));
        assert!(!coalesced[0].body.contains("/tmp/beta"));
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

    #[test]
    fn sound_selector_resolves_audio_directories_sequentially() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let sound_dir = temp_dir.path().join("sounds");
        fs::create_dir_all(&sound_dir).expect("sound dir should exist");
        fs::write(sound_dir.join("b.mp3"), "").expect("sound should be written");
        fs::write(sound_dir.join("a.aiff"), "").expect("sound should be written");
        fs::write(sound_dir.join("notes.txt"), "").expect("ignored file should be written");

        let profiles = std::collections::BTreeMap::from([(
            "itysl".to_string(),
            NotificationSoundProfile {
                completion: sound_dir.display().to_string(),
                needs_attention: "none".to_string(),
                cycle: NotificationSoundCycle::Sequential,
            },
        )]);
        let mut selector = super::NotificationSoundSelector::from_profiles(
            "itysl",
            &profiles,
            Some(temp_dir.path()),
        );

        assert_eq!(
            selector.resolve(NotificationKind::Completion),
            super::ResolvedNotificationSound::File(sound_dir.join("a.aiff"))
        );
        assert_eq!(
            selector.resolve(NotificationKind::Completion),
            super::ResolvedNotificationSound::File(sound_dir.join("b.mp3"))
        );
        assert_eq!(
            selector.resolve(NotificationKind::NeedsAttention),
            super::ResolvedNotificationSound::None
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

        fn send(
            &self,
            delivery: &super::NotificationDelivery<'_>,
        ) -> Result<(), NotificationError> {
            self.calls.borrow_mut().push(format!(
                "{}:{}",
                self.name,
                delivery.request.kind.label()
            ));
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
        let mut dispatcher = NotificationDispatcher::new(vec![
            Box::new(FakeBackend::new("primary", true, Rc::clone(&calls))),
            Box::new(FakeBackend::new("fallback", false, Rc::clone(&calls))),
        ]);
        let request = super::NotificationRequest {
            pane_id: "alpha:claude".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::Completion,
            title: "Agent ready: claude-main".to_string(),
            subtitle: "claude-main".to_string(),
            body: "The agent returned to an idle state.".to_string(),
            audible: true,
            window_target: Some("alpha:0".to_string()),
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
        let mut dispatcher = NotificationDispatcher::new(vec![Box::new(FakeBackend::new(
            "primary",
            true,
            Rc::clone(&calls),
        ))]);
        let request = super::NotificationRequest {
            pane_id: "alpha:claude".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::NeedsAttention,
            title: "Needs attention: claude-main".to_string(),
            subtitle: "claude-main".to_string(),
            body: "The agent is waiting for input or intervention.".to_string(),
            audible: true,
            window_target: Some("alpha:0".to_string()),
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
        let dispatcher = build_notification_dispatcher(
            &[
                NotificationBackendName::OsaScript,
                NotificationBackendName::NotifySend,
            ],
            "default",
            &Default::default(),
            None,
            None,
            None,
        );
        assert_eq!(
            dispatcher.backend_names(),
            vec!["osascript".to_string(), "notify-send".to_string()]
        );
    }

    #[test]
    fn alerter_click_selects_window_and_pane_without_blocking_dispatch() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let alerter = temp_dir.path().join("alerter");
        let tmux = temp_dir.path().join("tmux");
        let afplay = temp_dir.path().join("afplay");
        let alerter_args = temp_dir.path().join("alerter-args.txt");
        let tmux_log = temp_dir.path().join("tmux.txt");
        let afplay_log = temp_dir.path().join("afplay.txt");
        let diagnostic_log = temp_dir.path().join("latest.log");
        let sound = temp_dir.path().join("sound.aiff");
        fs::write(&sound, "").expect("sound should exist");

        write_executable(
            &alerter,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\nprintf '@CONTENTCLICKED\\n'\n",
                alerter_args.display()
            ),
        );
        write_executable(
            &tmux,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nif [ \"$3\" = \"display-message\" ]; then printf '$session-1\\t@1\\n'; fi\n",
                tmux_log.display()
            ),
        );
        write_executable(
            &afplay,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$1\" > \"{}\"\n",
                afplay_log.display()
            ),
        );

        let backend = super::AlerterNotificationBackend::with_programs(
            &alerter,
            &tmux,
            &afplay,
            Some(temp_dir.path().join("tmux.sock")),
            Some(diagnostic_log.clone()),
        );
        let request = super::NotificationRequest {
            pane_id: "%7".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::NeedsAttention,
            title: "Foreman: needs attention".to_string(),
            subtitle: "claude-main".to_string(),
            body: "alpha / @1 \"agents\" - Waiting for input".to_string(),
            audible: true,
            window_target: Some("@1".to_string()),
            workspace_path: Some(PathBuf::from("/tmp/alpha")),
        };

        backend
            .send(&super::NotificationDelivery {
                request: &request,
                sound: super::ResolvedNotificationSound::File(sound.clone()),
            })
            .expect("alerter should dispatch in the background");

        wait_for_file_contains(&alerter_args, "--actions Open tmux pane");
        wait_for_file_contains(&afplay_log, &sound.display().to_string());
        wait_for_file_contains(&tmux_log, "select-pane -t %7");
        let tmux_contents = fs::read_to_string(tmux_log).expect("tmux log should be readable");
        assert!(tmux_contents.contains("-S"));
        assert!(tmux_contents.contains("display-message -p -t %7"));
        assert!(tmux_contents.contains("switch-client -t $session-1"));
        assert!(tmux_contents.contains("select-window -t @1"));
        let diagnostics =
            fs::read_to_string(diagnostic_log).expect("diagnostic log should be written");
        assert!(diagnostics.contains("notification_alerter_completed"));
        assert!(diagnostics.contains("notification_alerter_focus_requested"));
        assert!(diagnostics.contains("notification_tmux_command"));
    }

    #[test]
    fn dispatcher_falls_back_when_alerter_exits_immediately() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let afplay = temp_dir.path().join("afplay");
        let capture_file = temp_dir.path().join("fallback.txt");
        let diagnostic_log = temp_dir.path().join("latest.log");

        write_executable(&afplay, "#!/bin/sh\nexit 0\n");
        let fallback_script = temp_dir.path().join("fallback.sh");
        write_executable(
            &fallback_script,
            &format!(
                "#!/bin/sh\nprintf '%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" > \"{}\"\n",
                capture_file.display()
            ),
        );

        let mut dispatcher = NotificationDispatcher::new(vec![
            Box::new(super::AlerterNotificationBackend::with_programs(
                "false",
                "tmux",
                &afplay,
                None,
                Some(diagnostic_log.clone()),
            )),
            Box::new(super::CommandNotificationBackend::new(
                "fallback",
                &fallback_script,
                std::iter::empty::<String>(),
            )),
        ]);
        let request = super::NotificationRequest {
            pane_id: "%7".into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::Completion,
            title: "Foreman: agent ready".to_string(),
            subtitle: "claude-main".to_string(),
            body: "alpha / @1 \"agents\" - Returned to idle".to_string(),
            audible: true,
            window_target: Some("@1".to_string()),
            workspace_path: Some(PathBuf::from("/tmp/alpha")),
        };

        let receipt = dispatcher
            .dispatch(&request)
            .expect("fallback backend should be used");

        assert_eq!(receipt.backend_name, "fallback");
        let capture = fs::read_to_string(capture_file).expect("fallback should capture request");
        assert_eq!(capture.trim(), "completion|Foreman: agent ready");
        let diagnostics =
            fs::read_to_string(diagnostic_log).expect("diagnostic log should be written");
        assert!(diagnostics.contains("notification_alerter_completed"));
        assert!(diagnostics.contains("status=exit status: 1"));
    }

    #[test]
    fn alerter_click_focuses_real_tmux_pane() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let socket = temp_dir.path().join("tmux.sock");
        run_tmux(&socket, &["start-server"]);
        run_tmux(
            &socket,
            &[
                "new-session",
                "-d",
                "-s",
                "notify-source",
                "sh -lc 'sleep 60'",
            ],
        );
        let target_pane = run_tmux(
            &socket,
            &[
                "new-session",
                "-d",
                "-P",
                "-F",
                "#{pane_id}",
                "-s",
                "notify-target",
                "sh -lc 'sleep 60'",
            ],
        );
        let other_pane = run_tmux(
            &socket,
            &[
                "split-window",
                "-d",
                "-P",
                "-F",
                "#{pane_id}",
                "-t",
                "notify-target",
                "sh -lc 'sleep 60'",
            ],
        );
        run_tmux(&socket, &["select-pane", "-t", &other_pane]);
        assert_eq!(
            run_tmux(
                &socket,
                &["display-message", "-p", "-t", "notify-target", "#{pane_id}",],
            ),
            other_pane
        );

        let alerter = temp_dir.path().join("alerter");
        let afplay = temp_dir.path().join("afplay");
        let afplay_log = temp_dir.path().join("afplay.txt");
        let sound = temp_dir.path().join("sound.aiff");
        fs::write(&sound, "").expect("sound should exist");
        write_executable(&alerter, "#!/bin/sh\nprintf '@CONTENTCLICKED\\n'\n");
        write_executable(
            &afplay,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$1\" > \"{}\"\n",
                afplay_log.display()
            ),
        );

        let backend = super::AlerterNotificationBackend::with_programs(
            &alerter,
            "tmux",
            &afplay,
            Some(socket.clone()),
            None,
        );
        let request = super::NotificationRequest {
            pane_id: target_pane.clone().into(),
            pane_title: "claude-main".to_string(),
            kind: NotificationKind::NeedsAttention,
            title: "Foreman: needs attention".to_string(),
            subtitle: "claude-main".to_string(),
            body: "notify-target / @1 \"agents\" - Waiting for input".to_string(),
            audible: true,
            window_target: None,
            workspace_path: Some(PathBuf::from("/tmp/alpha")),
        };
        backend
            .send(&super::NotificationDelivery {
                request: &request,
                sound: super::ResolvedNotificationSound::File(sound.clone()),
            })
            .expect("alerter should dispatch in the background");
        wait_for_file_contains(&afplay_log, &sound.display().to_string());
        for _ in 0..100 {
            if run_tmux(
                &socket,
                &["display-message", "-p", "-t", "notify-target", "#{pane_id}"],
            ) == target_pane
            {
                run_tmux(&socket, &["kill-server"]);
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let active = run_tmux(
            &socket,
            &["display-message", "-p", "-t", "notify-target", "#{pane_id}"],
        );
        run_tmux(&socket, &["kill-server"]);
        panic!("expected active pane {target_pane}, got {active}");
    }
}
