use clap::ValueEnum;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::AppPaths;
use crate::sources::{unix_ms_now, SourceId};

pub const SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION: u16 = 1;
const GHOSTTY_CAPTURE_SEPARATOR: char = '\u{001e}';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum DisplayProviderKind {
    Ghostty,
}

impl DisplayProviderKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ghostty => "ghostty",
        }
    }
}

impl std::fmt::Display for DisplayProviderKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.label())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayIdentity {
    pub terminal_uuid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_title: Option<String>,
}

impl DisplayIdentity {
    pub fn validate(&self) -> Result<(), DisplayError> {
        if self.terminal_uuid.trim().is_empty() {
            return Err(DisplayError::new(
                "source.display.invalid-identity",
                "Ghostty terminal UUID must not be empty",
                false,
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDisplayRegistration {
    pub source_id: String,
    pub provider: DisplayProviderKind,
    pub identity: DisplayIdentity,
    pub captured_at_unix_ms: u128,
    pub updated_at_unix_ms: u128,
    pub ownership_handle: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayActivationResponse {
    pub attempted: bool,
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<DisplayProviderKind>,
    #[serde(default)]
    pub fallback_attempted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayHealth {
    pub source_id: String,
    pub status: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration: Option<SourceDisplayRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl DisplayError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }
}

impl std::fmt::Display for DisplayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for DisplayError {}

impl From<io::Error> for DisplayError {
    fn from(error: io::Error) -> Self {
        Self::new(
            "source.display.registry-io",
            format!("display registry I/O failed: {error}"),
            true,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisplayRegistryEnvelope {
    schema_version: u16,
    #[serde(default)]
    registrations: BTreeMap<String, SourceDisplayRegistration>,
}

impl Default for DisplayRegistryEnvelope {
    fn default() -> Self {
        Self {
            schema_version: SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION,
            registrations: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceDisplayRegistry {
    path: PathBuf,
}

impl SourceDisplayRegistry {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn for_paths(paths: &AppPaths) -> Self {
        Self::for_log_dir(&paths.log_dir)
    }

    pub fn for_log_dir(log_dir: &Path) -> Self {
        let state_dir = log_dir.parent().unwrap_or(log_dir);
        Self::new(state_dir.join("source-displays.json"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get(
        &self,
        source_id: &SourceId,
    ) -> Result<Option<SourceDisplayRegistration>, DisplayError> {
        Ok(self.load()?.registrations.get(source_id.as_str()).cloned())
    }

    pub fn list(&self) -> Result<Vec<SourceDisplayRegistration>, DisplayError> {
        Ok(self.load()?.registrations.into_values().collect())
    }

    pub fn register(
        &self,
        source_id: &SourceId,
        provider: DisplayProviderKind,
        identity: DisplayIdentity,
        now_ms: u128,
    ) -> Result<SourceDisplayRegistration, DisplayError> {
        identity.validate()?;
        let _lock = RegistryLock::acquire(&self.lock_path())?;
        let mut envelope = self.load()?;
        let registration = SourceDisplayRegistration {
            source_id: source_id.as_str().to_string(),
            provider,
            identity,
            captured_at_unix_ms: now_ms,
            updated_at_unix_ms: now_ms,
            ownership_handle: new_opaque_handle()?,
        };
        envelope
            .registrations
            .insert(source_id.as_str().to_string(), registration.clone());
        self.atomic_write(&envelope)?;
        Ok(registration)
    }

    pub fn unregister(
        &self,
        source_id: &SourceId,
        ownership_handle: &str,
    ) -> Result<bool, DisplayError> {
        let _lock = RegistryLock::acquire(&self.lock_path())?;
        let mut envelope = self.load()?;
        let Some(current) = envelope.registrations.get(source_id.as_str()) else {
            return Ok(false);
        };
        if current.ownership_handle != ownership_handle {
            return Err(DisplayError::new(
                "source.display.ownership-mismatch",
                format!(
                    "display registration for source {} has a different ownership handle; list registrations and retry with the current handle",
                    source_id
                ),
                false,
            ));
        }
        envelope.registrations.remove(source_id.as_str());
        self.atomic_write(&envelope)?;
        Ok(true)
    }

    fn load(&self) -> Result<DisplayRegistryEnvelope, DisplayError> {
        let payload = match fs::read_to_string(&self.path) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(DisplayRegistryEnvelope::default())
            }
            Err(error) => return Err(error.into()),
        };
        let envelope: DisplayRegistryEnvelope =
            serde_json::from_str(&payload).map_err(|error| {
                DisplayError::new(
                    "source.display.registry-invalid",
                    format!(
                        "display registry {} is not valid JSON: {error}",
                        self.path.display()
                    ),
                    false,
                )
            })?;
        if envelope.schema_version != SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION {
            return Err(DisplayError::new(
                "source.display.registry-schema-unsupported",
                format!(
                    "display registry {} uses unsupported schema version {}",
                    self.path.display(),
                    envelope.schema_version
                ),
                false,
            ));
        }
        for (key, registration) in &envelope.registrations {
            if key != &registration.source_id {
                return Err(DisplayError::new(
                    "source.display.registry-source-id-mismatch",
                    format!(
                        "display registry entry {key} contains source id {}",
                        registration.source_id
                    ),
                    false,
                ));
            }
            registration.identity.validate()?;
        }
        Ok(envelope)
    }

    fn atomic_write(&self, envelope: &DisplayRegistryEnvelope) -> Result<(), DisplayError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp_path = self.path.with_extension(format!(
            "json.{}.{}.tmp",
            std::process::id(),
            new_opaque_handle()?
        ));
        let payload = serde_json::to_vec_pretty(envelope).map_err(|error| {
            DisplayError::new(
                "source.display.registry-invalid",
                format!("failed to encode display registry: {error}"),
                false,
            )
        })?;
        {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(&payload)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }

    fn lock_path(&self) -> PathBuf {
        self.path.with_extension("lock")
    }
}

struct RegistryLock {
    file: fs::File,
}

impl RegistryLock {
    fn acquire(path: &Path) -> Result<Self, DisplayError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        file.lock_exclusive().map_err(DisplayError::from)?;
        Ok(Self { file })
    }
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn new_opaque_handle() -> Result<String, DisplayError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        DisplayError::new(
            "source.display.registry-entropy",
            format!("failed to generate a display registration ownership handle: {error}"),
            true,
        )
    })?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub trait DisplayProviderAdapter {
    fn capture(&self, provider: DisplayProviderKind) -> Result<DisplayIdentity, DisplayError>;
    fn activate(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError>;
    fn check(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError>;
}

trait AppleScriptExecutor: Send + Sync {
    fn run(&self, script: &str, args: &[&str]) -> Result<String, DisplayError>;
}

struct ProcessAppleScriptExecutor;

impl AppleScriptExecutor for ProcessAppleScriptExecutor {
    fn run(&self, script: &str, args: &[&str]) -> Result<String, DisplayError> {
        let mut child = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .arg("--")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                DisplayError::new(
                    "source.display.provider-unavailable",
                    format!("failed to run Ghostty AppleScript through osascript: {error}"),
                    true,
                )
            })?;
        let timeout = Duration::from_secs(2);
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => {
                    let output = child.wait_with_output().map_err(|error| {
                        DisplayError::new(
                            "source.display.provider-unavailable",
                            format!("failed to collect Ghostty AppleScript output: {error}"),
                            true,
                        )
                    })?;
                    if output.status.success() {
                        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    }
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    return Err(DisplayError::new(
                        "source.display.unavailable",
                        if stderr.is_empty() {
                            format!(
                                "Ghostty rejected the AppleScript request with status {}; confirm Ghostty 1.3+ is running and grant Foreman Automation access in System Settings > Privacy & Security > Automation",
                                output.status
                            )
                        } else {
                            format!(
                                "Ghostty display is unavailable: {stderr}. Confirm the registered terminal is open and grant Foreman Automation access in System Settings > Privacy & Security > Automation"
                            )
                        },
                        true,
                    ));
                }
                Ok(None) if started.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DisplayError::new(
                        "source.display.provider-timeout",
                        "Ghostty AppleScript timed out after 2000ms; confirm Ghostty is running and resolve macOS Automation access in System Settings > Privacy & Security > Automation",
                        true,
                    ));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DisplayError::new(
                        "source.display.provider-unavailable",
                        format!("failed to wait for Ghostty AppleScript: {error}"),
                        true,
                    ));
                }
            }
        }
    }
}

pub struct SystemDisplayProvider {
    macos: bool,
    executor: Box<dyn AppleScriptExecutor>,
}

impl Default for SystemDisplayProvider {
    fn default() -> Self {
        Self {
            macos: cfg!(target_os = "macos"),
            executor: Box::new(ProcessAppleScriptExecutor),
        }
    }
}

impl SystemDisplayProvider {
    fn ensure_platform(&self, provider: DisplayProviderKind) -> Result<(), DisplayError> {
        if self.macos {
            return Ok(());
        }
        Err(DisplayError::new(
            "source.display.platform-unsupported",
            format!(
                "{} display capture and focus require macOS and Ghostty 1.3+; use an activation_command fallback on this platform",
                provider.label()
            ),
            false,
        ))
    }

    #[cfg(test)]
    fn with_executor(macos: bool, executor: Box<dyn AppleScriptExecutor>) -> Self {
        Self { macos, executor }
    }
}

impl DisplayProviderAdapter for SystemDisplayProvider {
    fn capture(&self, provider: DisplayProviderKind) -> Result<DisplayIdentity, DisplayError> {
        self.ensure_platform(provider)?;
        match provider {
            DisplayProviderKind::Ghostty => {
                let output = self.executor.run(ghostty_capture_script(), &[])?;
                parse_ghostty_capture(&output)
            }
        }
    }

    fn activate(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
        self.ensure_platform(registration.provider)?;
        registration.identity.validate()?;
        match registration.provider {
            DisplayProviderKind::Ghostty => self
                .executor
                .run(
                    ghostty_focus_script(),
                    &[registration.identity.terminal_uuid.as_str()],
                )
                .map(|_| ()),
        }
    }

    fn check(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
        self.ensure_platform(registration.provider)?;
        registration.identity.validate()?;
        match registration.provider {
            DisplayProviderKind::Ghostty => self
                .executor
                .run(
                    ghostty_check_script(),
                    &[registration.identity.terminal_uuid.as_str()],
                )
                .map(|_| ()),
        }
    }
}

fn parse_ghostty_capture(output: &str) -> Result<DisplayIdentity, DisplayError> {
    let output = output.trim_end_matches(['\r', '\n']);
    let mut fields = output.splitn(4, GHOSTTY_CAPTURE_SEPARATOR);
    let terminal_uuid = fields.next().unwrap_or_default().to_string();
    let tab_id = non_empty(fields.next());
    let window_id = non_empty(fields.next());
    let diagnostic_title = non_empty(fields.next());
    let identity = DisplayIdentity {
        terminal_uuid,
        tab_id,
        window_id,
        diagnostic_title,
    };
    identity.validate()?;
    Ok(identity)
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|value| !value.is_empty()).map(str::to_string)
}

pub fn ghostty_capture_script() -> &'static str {
    r#"tell application "Ghostty"
    if (count of windows) is 0 then error "Ghostty has no open windows" number 1728
    set targetWindow to front window
    set targetTab to selected tab of targetWindow
    if targetTab is missing value then error "Ghostty front window has no selected tab" number 1728
    set targetTerminal to focused terminal of targetTab
    if targetTerminal is missing value then error "Ghostty selected tab has no focused terminal" number 1728
    set fieldSeparator to ASCII character 30
    return (id of targetTerminal) & fieldSeparator & (id of targetTab) & fieldSeparator & (id of targetWindow) & fieldSeparator & (name of targetTerminal)
end tell"#
}

pub fn ghostty_focus_script() -> &'static str {
    r#"on run argv
    set targetID to item 1 of argv
    tell application "Ghostty"
        set matchingTerminals to every terminal whose id is targetID
        if (count of matchingTerminals) is 0 then error "registered Ghostty terminal is closed or missing" number 1728
        set targetTerminal to item 1 of matchingTerminals
        focus targetTerminal
        activate
        return id of targetTerminal
    end tell
end run"#
}

pub fn ghostty_check_script() -> &'static str {
    r#"on run argv
    set targetID to item 1 of argv
    tell application "Ghostty"
        set matchingTerminals to every terminal whose id is targetID
        if (count of matchingTerminals) is 0 then error "registered Ghostty terminal is closed or missing" number 1728
        return id of item 1 of matchingTerminals
    end tell
end run"#
}

pub trait ActivationCommandRunner {
    fn run(
        &self,
        command: &str,
        source_id: &str,
        pane_id: &str,
        timeout: Duration,
    ) -> Result<(), DisplayError>;
}

pub struct SystemActivationCommandRunner {
    login_shell: bool,
}

impl SystemActivationCommandRunner {
    pub fn login_shell() -> Self {
        Self { login_shell: true }
    }

    pub fn non_login_shell() -> Self {
        Self { login_shell: false }
    }
}

impl ActivationCommandRunner for SystemActivationCommandRunner {
    fn run(
        &self,
        command: &str,
        source_id: &str,
        pane_id: &str,
        timeout: Duration,
    ) -> Result<(), DisplayError> {
        let expanded_command = expand_activation_command(command, source_id, pane_id);
        let mut child = Command::new("sh")
            .arg(if self.login_shell { "-lc" } else { "-c" })
            .arg(expanded_command)
            .env("FOREMAN_SOURCE_ID", source_id)
            .env("FOREMAN_PANE_ID", pane_id)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                DisplayError::new(
                    "source.display.activation-command-failed",
                    format!("failed to run activation command: {error}"),
                    true,
                )
            })?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => {
                    let _ = child.wait_with_output();
                    return Ok(());
                }
                Ok(Some(status)) => {
                    let output = child.wait_with_output().ok();
                    let stderr = output
                        .as_ref()
                        .map(|output| String::from_utf8_lossy(&output.stderr).trim().to_string())
                        .unwrap_or_default();
                    let stdout = output
                        .as_ref()
                        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
                        .unwrap_or_default();
                    let detail = if stderr.is_empty() { stdout } else { stderr };
                    let message = if detail.is_empty() {
                        format!("activation command exited with status {status}")
                    } else {
                        format!("activation command exited with status {status}: {detail}")
                    };
                    return Err(DisplayError::new(
                        "source.display.activation-command-failed",
                        message,
                        true,
                    ));
                }
                Ok(None) if started.elapsed() >= timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DisplayError::new(
                        "source.display.activation-command-timeout",
                        format!(
                            "activation command timed out after {}ms",
                            timeout.as_millis()
                        ),
                        true,
                    ));
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(error) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(DisplayError::new(
                        "source.display.activation-command-failed",
                        format!("failed to wait for activation command: {error}"),
                        true,
                    ));
                }
            }
        }
    }
}

fn expand_activation_command(template: &str, source_id: &str, pane_id: &str) -> String {
    template
        .replace("{source_id}", &shell_quote(source_id))
        .replace("{pane_id}", &shell_quote(pane_id))
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-' | ':' | '=' | '%')
    }) {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

pub fn activate_registered_display_or_fallback(
    registry: &SourceDisplayRegistry,
    provider: &dyn DisplayProviderAdapter,
    command_runner: &dyn ActivationCommandRunner,
    source_id: &SourceId,
    pane_id: &str,
    fallback_command: Option<&str>,
    fallback_timeout: Duration,
) -> Option<DisplayActivationResponse> {
    let registration = match registry.get(source_id) {
        Ok(registration) => registration,
        Err(error) => {
            return Some(run_fallback_after_error(
                command_runner,
                source_id,
                pane_id,
                fallback_command,
                fallback_timeout,
                None,
                error,
            ))
        }
    };

    if let Some(registration) = registration {
        match provider.activate(&registration) {
            Ok(()) => {
                return Some(DisplayActivationResponse {
                    attempted: true,
                    ok: true,
                    provider: Some(registration.provider),
                    fallback_attempted: false,
                    code: None,
                    message: None,
                })
            }
            Err(error) => {
                return Some(run_fallback_after_error(
                    command_runner,
                    source_id,
                    pane_id,
                    fallback_command,
                    fallback_timeout,
                    Some(registration.provider),
                    error,
                ))
            }
        }
    }

    let fallback_command = fallback_command.filter(|command| !command.trim().is_empty())?;
    Some(run_fallback_only(
        command_runner,
        source_id,
        pane_id,
        fallback_command,
        fallback_timeout,
    ))
}

fn run_fallback_only(
    command_runner: &dyn ActivationCommandRunner,
    source_id: &SourceId,
    pane_id: &str,
    command: &str,
    timeout: Duration,
) -> DisplayActivationResponse {
    match command_runner.run(command, source_id.as_str(), pane_id, timeout) {
        Ok(()) => DisplayActivationResponse {
            attempted: true,
            ok: true,
            provider: None,
            fallback_attempted: true,
            code: None,
            message: None,
        },
        Err(error) => DisplayActivationResponse {
            attempted: true,
            ok: false,
            provider: None,
            fallback_attempted: true,
            code: Some(error.code),
            message: Some(error.message),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn run_fallback_after_error(
    command_runner: &dyn ActivationCommandRunner,
    source_id: &SourceId,
    pane_id: &str,
    fallback_command: Option<&str>,
    timeout: Duration,
    provider_kind: Option<DisplayProviderKind>,
    provider_error: DisplayError,
) -> DisplayActivationResponse {
    let Some(command) = fallback_command.filter(|command| !command.trim().is_empty()) else {
        return DisplayActivationResponse {
            attempted: true,
            ok: false,
            provider: provider_kind,
            fallback_attempted: false,
            code: Some(provider_error.code),
            message: Some(provider_error.message),
        };
    };
    match command_runner.run(command, source_id.as_str(), pane_id, timeout) {
        Ok(()) => DisplayActivationResponse {
            attempted: true,
            ok: true,
            provider: provider_kind,
            fallback_attempted: true,
            code: Some("source.display.fallback-used".to_string()),
            message: Some(format!(
                "{}: {}; activation command fallback succeeded",
                provider_error.code, provider_error.message
            )),
        },
        Err(fallback_error) => DisplayActivationResponse {
            attempted: true,
            ok: false,
            provider: provider_kind,
            fallback_attempted: true,
            code: Some(fallback_error.code.clone()),
            message: Some(format!(
                "{}: {}; fallback failed: {}: {}",
                provider_error.code,
                provider_error.message,
                fallback_error.code,
                fallback_error.message
            )),
        },
    }
}

pub fn display_health(
    registry: &SourceDisplayRegistry,
    provider: &dyn DisplayProviderAdapter,
    source_id: &SourceId,
    probe: bool,
) -> DisplayHealth {
    let registration = match registry.get(source_id) {
        Ok(Some(registration)) => registration,
        Ok(None) => {
            return DisplayHealth {
                source_id: source_id.as_str().to_string(),
                status: "not-registered".to_string(),
                code: "source.display.not-registered".to_string(),
                message: format!(
                    "source {} has no local display registration; capture one with `foreman sources display capture {} --provider ghostty` or keep using activation_command",
                    source_id, source_id
                ),
                registration: None,
            }
        }
        Err(error) => {
            return DisplayHealth {
                source_id: source_id.as_str().to_string(),
                status: "unavailable".to_string(),
                code: error.code,
                message: error.message,
                registration: None,
            }
        }
    };
    if !probe {
        return DisplayHealth {
            source_id: source_id.as_str().to_string(),
            status: "registered".to_string(),
            code: "source.display.registered".to_string(),
            message: format!(
                "{} display registration is present; run sources display doctor to probe the exact terminal",
                registration.provider.label()
            ),
            registration: Some(registration),
        };
    }
    match provider.check(&registration) {
        Ok(()) => DisplayHealth {
            source_id: source_id.as_str().to_string(),
            status: "ok".to_string(),
            code: "source.display.ok".to_string(),
            message: format!(
                "registered {} terminal {} is available",
                registration.provider.label(),
                registration.identity.terminal_uuid
            ),
            registration: Some(registration),
        },
        Err(error) => DisplayHealth {
            source_id: source_id.as_str().to_string(),
            status: "unavailable".to_string(),
            code: error.code,
            message: error.message,
            registration: Some(registration),
        },
    }
}

pub fn capture_and_register(
    registry: &SourceDisplayRegistry,
    provider_adapter: &dyn DisplayProviderAdapter,
    source_id: &SourceId,
    provider: DisplayProviderKind,
) -> Result<SourceDisplayRegistration, DisplayError> {
    let identity = provider_adapter.capture(provider)?;
    registry.register(source_id, provider, identity, unix_ms_now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn identity(uuid: &str) -> DisplayIdentity {
        DisplayIdentity {
            terminal_uuid: uuid.to_string(),
            tab_id: Some("tab-1".to_string()),
            window_id: Some("window-1".to_string()),
            diagnostic_title: Some("duplicate title".to_string()),
        }
    }

    #[test]
    fn registry_path_is_owned_by_the_state_directory_above_logs() {
        let registry = SourceDisplayRegistry::for_log_dir(Path::new("/tmp/foreman/logs"));
        assert_eq!(
            registry.path(),
            Path::new("/tmp/foreman/source-displays.json")
        );
    }

    #[test]
    fn registration_serializes_typed_provider_and_exact_identity() {
        let registration = SourceDisplayRegistration {
            source_id: "remote-dev".to_string(),
            provider: DisplayProviderKind::Ghostty,
            identity: identity("terminal-uuid"),
            captured_at_unix_ms: 100,
            updated_at_unix_ms: 101,
            ownership_handle: "opaque".to_string(),
        };
        let value = serde_json::to_value(&registration).unwrap();
        assert_eq!(value["provider"], "ghostty");
        assert_eq!(value["identity"]["terminalUuid"], "terminal-uuid");
        assert_eq!(value["identity"]["diagnosticTitle"], "duplicate title");
    }

    #[test]
    fn replacement_invalidates_old_ownership_and_old_cleanup_cannot_delete_it() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
        let source_id = SourceId::new("remote-dev");
        let first = registry
            .register(
                &source_id,
                DisplayProviderKind::Ghostty,
                identity("one"),
                10,
            )
            .unwrap();
        let second = registry
            .register(
                &source_id,
                DisplayProviderKind::Ghostty,
                identity("two"),
                20,
            )
            .unwrap();

        let error = registry
            .unregister(&source_id, &first.ownership_handle)
            .expect_err("old owner must not delete replacement");
        assert_eq!(error.code, "source.display.ownership-mismatch");
        assert_eq!(
            registry
                .get(&source_id)
                .unwrap()
                .unwrap()
                .identity
                .terminal_uuid,
            "two"
        );
        assert!(registry
            .unregister(&source_id, &second.ownership_handle)
            .unwrap());
        assert!(registry.get(&source_id).unwrap().is_none());
    }

    type AppleScriptCalls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

    #[derive(Clone)]
    struct RecordingAppleScript {
        output: Result<String, DisplayError>,
        calls: AppleScriptCalls,
    }

    impl AppleScriptExecutor for RecordingAppleScript {
        fn run(&self, script: &str, args: &[&str]) -> Result<String, DisplayError> {
            self.calls.lock().unwrap().push((
                script.to_string(),
                args.iter().map(|value| (*value).to_string()).collect(),
            ));
            self.output.clone()
        }
    }

    #[test]
    fn ghostty_capture_parses_stable_ids_and_title_as_diagnostic_only() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let executor = RecordingAppleScript {
            output: Ok(
                "terminal-exact\u{001e}tab-exact\u{001e}window-exact\u{001e}same title\n"
                    .to_string(),
            ),
            calls: calls.clone(),
        };
        let provider = SystemDisplayProvider::with_executor(true, Box::new(executor));
        let captured = provider.capture(DisplayProviderKind::Ghostty).unwrap();
        assert_eq!(captured.terminal_uuid, "terminal-exact");
        assert_eq!(captured.tab_id.as_deref(), Some("tab-exact"));
        assert_eq!(captured.window_id.as_deref(), Some("window-exact"));
        assert_eq!(captured.diagnostic_title.as_deref(), Some("same title"));
        assert!(calls.lock().unwrap()[0].0.contains("focused terminal"));
    }

    #[test]
    fn ghostty_focus_passes_only_exact_terminal_uuid_and_never_uses_title() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let executor = RecordingAppleScript {
            output: Ok("terminal-exact\n".to_string()),
            calls: calls.clone(),
        };
        let provider = SystemDisplayProvider::with_executor(true, Box::new(executor));
        let registration = SourceDisplayRegistration {
            source_id: "remote-dev".to_string(),
            provider: DisplayProviderKind::Ghostty,
            identity: identity("terminal-exact"),
            captured_at_unix_ms: 1,
            updated_at_unix_ms: 1,
            ownership_handle: "handle".to_string(),
        };
        provider.activate(&registration).unwrap();
        let calls = calls.lock().unwrap();
        assert_eq!(calls[0].1, vec!["terminal-exact"]);
        assert!(calls[0].0.contains("every terminal whose id is targetID"));
        assert!(calls[0].0.contains("focus targetTerminal"));
        assert!(!calls[0].0.contains("name of"));
    }

    #[test]
    fn unsupported_platform_is_typed_and_does_not_run_applescript() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let executor = RecordingAppleScript {
            output: Ok(String::new()),
            calls: calls.clone(),
        };
        let provider = SystemDisplayProvider::with_executor(false, Box::new(executor));
        let error = provider
            .capture(DisplayProviderKind::Ghostty)
            .expect_err("non-macOS must fail clearly");
        assert_eq!(error.code, "source.display.platform-unsupported");
        assert!(calls.lock().unwrap().is_empty());
    }

    struct FakeProvider {
        activation: Result<(), DisplayError>,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl DisplayProviderAdapter for FakeProvider {
        fn capture(&self, _provider: DisplayProviderKind) -> Result<DisplayIdentity, DisplayError> {
            Ok(identity("captured"))
        }

        fn activate(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("provider:{}", registration.identity.terminal_uuid));
            self.activation.clone()
        }

        fn check(&self, _registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
            self.activation.clone()
        }
    }

    struct FakeCommandRunner {
        result: Result<(), DisplayError>,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl ActivationCommandRunner for FakeCommandRunner {
        fn run(
            &self,
            command: &str,
            _source_id: &str,
            _pane_id: &str,
            _timeout: Duration,
        ) -> Result<(), DisplayError> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("fallback:{command}"));
            self.result.clone()
        }
    }

    #[test]
    fn resolver_attempts_registered_identity_before_command_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
        let source_id = SourceId::new("remote-dev");
        registry
            .register(
                &source_id,
                DisplayProviderKind::Ghostty,
                identity("terminal-exact"),
                1,
            )
            .unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = FakeProvider {
            activation: Err(DisplayError::new(
                "source.display.unavailable",
                "closed terminal",
                true,
            )),
            calls: calls.clone(),
        };
        let runner = FakeCommandRunner {
            result: Ok(()),
            calls: calls.clone(),
        };

        let outcome = activate_registered_display_or_fallback(
            &registry,
            &provider,
            &runner,
            &source_id,
            "%42",
            Some("fallback-command"),
            Duration::from_secs(1),
        )
        .unwrap();

        assert!(outcome.ok);
        assert!(outcome.fallback_attempted);
        assert_eq!(
            outcome.code.as_deref(),
            Some("source.display.fallback-used")
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["provider:terminal-exact", "fallback:fallback-command"]
        );
    }

    #[test]
    fn stale_or_closed_target_is_actionable_without_a_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
        let source_id = SourceId::new("remote-dev");
        registry
            .register(
                &source_id,
                DisplayProviderKind::Ghostty,
                identity("closed"),
                1,
            )
            .unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let outcome = activate_registered_display_or_fallback(
            &registry,
            &FakeProvider {
                activation: Err(DisplayError::new(
                    "source.display.unavailable",
                    "registered Ghostty terminal is closed or missing",
                    true,
                )),
                calls: calls.clone(),
            },
            &FakeCommandRunner {
                result: Ok(()),
                calls,
            },
            &source_id,
            "%42",
            None,
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(outcome.attempted);
        assert!(!outcome.ok);
        assert_eq!(outcome.code.as_deref(), Some("source.display.unavailable"));
        assert!(outcome.message.unwrap().contains("closed or missing"));
    }

    #[test]
    fn activation_command_failure_preserves_stdout_diagnostic() {
        let error = SystemActivationCommandRunner::login_shell()
            .run(
                "printf 'activation target missing'; exit 7",
                "remote-dev",
                "%42",
                Duration::from_secs(1),
            )
            .expect_err("command should fail");

        assert_eq!(error.code, "source.display.activation-command-failed");
        assert!(error.message.contains("activation target missing"));
    }

    #[test]
    fn activation_command_only_configuration_remains_supported() {
        let dir = tempfile::tempdir().unwrap();
        let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
        let source_id = SourceId::new("legacy");
        let calls = Arc::new(Mutex::new(Vec::new()));
        let outcome = activate_registered_display_or_fallback(
            &registry,
            &FakeProvider {
                activation: Ok(()),
                calls: calls.clone(),
            },
            &FakeCommandRunner {
                result: Ok(()),
                calls: calls.clone(),
            },
            &source_id,
            "%7",
            Some("legacy-command"),
            Duration::from_secs(1),
        )
        .unwrap();
        assert!(outcome.ok);
        assert_eq!(outcome.provider, None);
        assert_eq!(*calls.lock().unwrap(), vec!["fallback:legacy-command"]);
    }
}
