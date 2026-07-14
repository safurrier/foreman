use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

use crate::sources::{unix_ms_now, SourceId};

pub const SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION: u16 = 1;
pub const DEFAULT_ACTIVATION_TIMEOUT: Duration = Duration::from_secs(2);
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

mod registry;
use registry::new_opaque_handle;
pub use registry::SourceDisplayRegistry;

pub trait DisplayProviderAdapter {
    fn capture(&self, provider: DisplayProviderKind) -> Result<DisplayIdentity, DisplayError>;
    fn activate(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError>;
    fn check(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError>;
}

mod command;
mod ghostty;

pub use command::{ActivationCommandRunner, SystemActivationCommandRunner};
#[cfg(test)]
use ghostty::AppleScriptExecutor;
pub use ghostty::{
    ghostty_capture_script, ghostty_check_script, ghostty_focus_script, SystemDisplayProvider,
};

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
mod tests;
