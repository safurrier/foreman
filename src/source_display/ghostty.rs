use super::{
    DisplayError, DisplayIdentity, DisplayProviderAdapter, DisplayProviderKind,
    SourceDisplayRegistration, GHOSTTY_CAPTURE_SEPARATOR,
};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub(super) trait AppleScriptExecutor: Send + Sync {
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
    pub(super) fn with_executor(macos: bool, executor: Box<dyn AppleScriptExecutor>) -> Self {
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
