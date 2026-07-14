use super::{new_opaque_handle, DisplayError};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

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
        let capture = CommandOutputCapture::new()?;
        let mut command = Command::new("sh");
        command
            .arg(if self.login_shell { "-lc" } else { "-c" })
            .arg(expanded_command)
            .env("FOREMAN_SOURCE_ID", source_id)
            .env("FOREMAN_PANE_ID", pane_id);
        capture.attach(&mut command)?;
        configure_process_group(&mut command);
        let mut child = command.spawn().map_err(|error| {
            DisplayError::new(
                "source.display.activation-command-failed",
                format!("failed to run activation command: {error}"),
                true,
            )
        })?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => return Ok(()),
                Ok(Some(status)) => {
                    let (stdout, stderr) = capture.read()?;
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
                    terminate_process_group(&mut child);
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
                    terminate_process_group(&mut child);
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

struct CommandOutputCapture {
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

impl CommandOutputCapture {
    fn new() -> Result<Self, DisplayError> {
        let stem = format!(
            "foreman-activation-{}-{}",
            std::process::id(),
            new_opaque_handle()?
        );
        let root = std::env::temp_dir();
        Ok(Self {
            stdout_path: root.join(format!("{stem}.stdout")),
            stderr_path: root.join(format!("{stem}.stderr")),
        })
    }

    fn attach(&self, command: &mut Command) -> Result<(), DisplayError> {
        let stdout = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.stdout_path)?;
        let stderr = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&self.stderr_path)?;
        command
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr));
        Ok(())
    }

    fn read(&self) -> Result<(String, String), DisplayError> {
        Ok((
            read_bounded_diagnostic(&self.stdout_path)?,
            read_bounded_diagnostic(&self.stderr_path)?,
        ))
    }
}

fn read_bounded_diagnostic(path: &Path) -> Result<String, DisplayError> {
    const MAX_DIAGNOSTIC_BYTES: usize = 4_096;
    let payload = fs::read(path)?;
    Ok(
        String::from_utf8_lossy(&payload[..payload.len().min(MAX_DIAGNOSTIC_BYTES)])
            .trim()
            .to_string(),
    )
}

impl Drop for CommandOutputCapture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.stdout_path);
        let _ = fs::remove_file(&self.stderr_path);
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

fn terminate_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", &format!("-{}", child.id())])
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
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
