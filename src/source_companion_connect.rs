use crate::source_companion::SOURCE_COMPANION_PROTOCOL_VERSION;
use crate::sources::SourceId;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectSshConfig {
    pub host: String,
    pub source_id: String,
    pub label: String,
    pub remote_port: u16,
    pub remote_bind_host: String,
    pub local_bind: String,
    pub allow_send: bool,
    pub token: Option<String>,
    pub activation_command: Option<String>,
    pub activation_timeout_ms: u64,
    pub remote_foreman: String,
    pub remote_config_file: Option<PathBuf>,
    pub ssh: String,
    pub extra_ssh_args: Vec<String>,
    pub replace: bool,
    pub no_remote_config: bool,
    pub json: bool,
    pub config_file: Option<PathBuf>,
    pub tmux_socket: Option<PathBuf>,
    pub tmux_server_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectSshSummary {
    pub ok: bool,
    pub action: String,
    pub source_id: String,
    pub label: String,
    pub remote_host_label: String,
    pub local_endpoint: String,
    pub remote_endpoint: String,
    pub remote_configured: bool,
    pub allow_send: bool,
}

pub fn connect_ssh(config: ConnectSshConfig) -> Result<ConnectSshSummary, String> {
    SourceId::validate(&config.source_id)?;
    validate_connect_ssh_config(&config)?;
    let token = config.token.clone().unwrap_or_else(generate_token);
    let ready_file = ready_file_path(&config.source_id);
    let _ = fs::remove_file(&ready_file);

    let mut companion = ChildGuard::new(start_companion_server(&config, &token, &ready_file)?);
    let local_endpoint = wait_for_ready_file(&ready_file, Duration::from_secs(10))?;
    let local_port = endpoint_port(&local_endpoint)?;

    let mut ssh = ChildGuard::new(start_reverse_tunnel(&config, local_port)?);
    let remote_endpoint = format!("{}:{}", config.remote_bind_host, config.remote_port);

    probe_remote_companion(&config, &remote_endpoint, &token)?;

    let mut remote_configured = false;
    if !config.no_remote_config {
        configure_remote_source(&config, &remote_endpoint, &token)?;
        remote_configured = true;
    }

    emit_ready(
        &config,
        &local_endpoint,
        &remote_endpoint,
        remote_configured,
    );

    loop {
        if let Some(status) = companion
            .child_mut()
            .try_wait()
            .map_err(|error| format!("failed to inspect companion server: {error}"))?
        {
            ssh.shutdown();
            return Err(format!(
                "companion server exited unexpectedly with {status}"
            ));
        }
        if let Some(status) = ssh
            .child_mut()
            .try_wait()
            .map_err(|error| format!("failed to inspect ssh tunnel: {error}"))?
        {
            companion.shutdown();
            return Err(format!(
                "ssh reverse tunnel exited unexpectedly with {status}"
            ));
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn start_companion_server(
    config: &ConnectSshConfig,
    token: &str,
    ready_file: &PathBuf,
) -> Result<Child, String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current foreman executable: {error}"))?;
    let mut command = Command::new(exe);
    push_runtime_args(&mut command, config);
    command
        .arg("companion")
        .arg("serve")
        .arg("--bind")
        .arg(&config.local_bind)
        .arg("--source-id")
        .arg(&config.source_id)
        .arg("--token")
        .arg(token)
        .arg("--ready-file")
        .arg(ready_file);
    if config.allow_send {
        command.arg("--allow-send");
    }
    command
        .arg("--activation-timeout-ms")
        .arg(config.activation_timeout_ms.to_string());
    if let Some(activation_command) = &config.activation_command {
        command.arg("--activation-command").arg(activation_command);
    }
    if config.json {
        command.arg("--json");
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start companion server: {error}"))
}

fn push_runtime_args(command: &mut Command, config: &ConnectSshConfig) {
    if let Some(config_file) = &config.config_file {
        command.arg("--config-file").arg(config_file);
    }
    if let Some(tmux_socket) = &config.tmux_socket {
        command.arg("--tmux-socket").arg(tmux_socket);
    }
    if let Some(tmux_server_name) = &config.tmux_server_name {
        command.arg("--tmux-server-name").arg(tmux_server_name);
    }
}

fn start_reverse_tunnel(config: &ConnectSshConfig, local_port: u16) -> Result<Child, String> {
    let mut command = Command::new(&config.ssh);
    command
        .arg("-N")
        .arg("-o")
        .arg("ControlMaster=no")
        .arg("-o")
        .arg("ControlPath=none")
        .arg("-o")
        .arg("ExitOnForwardFailure=yes");
    for arg in &config.extra_ssh_args {
        command.arg(arg);
    }
    command
        .arg("-R")
        .arg(format!(
            "{}:{}:127.0.0.1:{}",
            config.remote_bind_host, config.remote_port, local_port
        ))
        .arg(&config.host)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start ssh reverse tunnel: {error}"))
}

fn probe_remote_companion(
    config: &ConnectSshConfig,
    remote_endpoint: &str,
    token: &str,
) -> Result<(), String> {
    let started = Instant::now();
    let mut last_error = String::new();
    while started.elapsed() <= Duration::from_secs(10) {
        match run_remote_shell(config, &remote_probe_command(remote_endpoint, token)) {
            Ok(_) => return Ok(()),
            Err(error) => {
                last_error = error;
                thread::sleep(Duration::from_millis(250));
            }
        }
    }
    Err(format!(
        "reverse tunnel probe failed for {} through {}: {last_error}",
        remote_endpoint, config.host
    ))
}

fn remote_probe_command(remote_endpoint: &str, token: &str) -> String {
    let (host, port) = remote_endpoint
        .rsplit_once(':')
        .unwrap_or(("127.0.0.1", remote_endpoint));
    let script = format!(
        r#"import json, socket, sys
req = {{"protocolVersion": {protocol}, "requestId": "connect-ssh-probe", "token": {token}, "action": "agents", "allPanes": False}}
s = socket.create_connection(({host}, {port}), timeout=2)
s.settimeout(2)
s.sendall(json.dumps(req).encode() + b"\n")
line = b""
while not line.endswith(b"\n"):
    chunk = s.recv(65536)
    if not chunk:
        break
    line += chunk
resp = json.loads(line.decode())
if not resp.get("ok"):
    raise SystemExit(resp.get("errorMessage") or resp.get("errorCode") or "probe failed")
"#,
        protocol = SOURCE_COMPANION_PROTOCOL_VERSION,
        token = serde_json::to_string(token).unwrap_or_else(|_| "null".to_string()),
        host = serde_json::to_string(host).unwrap_or_else(|_| "\"127.0.0.1\"".to_string()),
        port = port,
    );
    format!("python3 -c {}", shell_quote(&script))
}

fn configure_remote_source(
    config: &ConnectSshConfig,
    remote_endpoint: &str,
    token: &str,
) -> Result<(), String> {
    let exists = remote_source_exists(config)?;
    if exists && !config.replace {
        return Err(format!(
            "remote source {} already exists on {}; pass --replace to overwrite it",
            config.source_id, config.host
        ));
    }
    if exists && config.replace {
        let remove = format!(
            "{} sources remove {} --json >/dev/null 2>&1 || true",
            remote_foreman_prefix(config),
            shell_quote(&config.source_id)
        );
        run_remote_shell(config, &remove)?;
    }

    let mut command = format!(
        "{} sources add companion {} --endpoint {} --label {} --token {} --json",
        remote_foreman_prefix(config),
        shell_quote(&config.source_id),
        shell_quote(remote_endpoint),
        shell_quote(&config.label),
        shell_quote(token)
    );
    if config.allow_send {
        command.push_str(" --allow-send");
    }
    run_remote_shell(config, &command).map(|_| ())
}

fn remote_source_exists(config: &ConnectSshConfig) -> Result<bool, String> {
    let command = format!("{} sources list --json", remote_foreman_prefix(config));
    let output = run_remote_shell(config, &command)?;
    let value: Value = serde_json::from_str(&output)
        .map_err(|error| format!("remote sources list returned invalid JSON: {error}"))?;
    let sources = value
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| "remote sources list JSON did not contain a sources array".to_string())?;
    Ok(sources.iter().any(|source| {
        source
            .get("id")
            .and_then(Value::as_str)
            .is_some_and(|id| id == config.source_id)
    }))
}

fn remote_foreman_prefix(config: &ConnectSshConfig) -> String {
    let mut parts = vec![shell_quote(&config.remote_foreman)];
    if let Some(path) = &config.remote_config_file {
        parts.push("--config-file".to_string());
        parts.push(shell_quote_os(path.as_os_str()));
    }
    parts.join(" ")
}

fn run_remote_shell(config: &ConnectSshConfig, remote_command: &str) -> Result<String, String> {
    let mut command = Command::new(&config.ssh);
    for arg in &config.extra_ssh_args {
        command.arg(arg);
    }
    command.arg(&config.host).arg(remote_command);
    let output = run_with_timeout(command, Duration::from_secs(15))?;
    if !output.status.success() {
        return Err(format!(
            "remote ssh command failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("remote ssh command returned non-UTF8 output: {error}"))
}

fn run_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to run remote ssh command: {error}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child.wait_with_output().map_err(|error| {
                    format!("failed to collect remote ssh command output: {error}")
                });
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "remote ssh command timed out after {}ms",
                    timeout.as_millis()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed to inspect remote ssh command: {error}"));
            }
        }
    }
}

fn wait_for_ready_file(path: &PathBuf, timeout: Duration) -> Result<String, String> {
    let started = Instant::now();
    loop {
        match fs::read_to_string(path) {
            Ok(value) if !value.trim().is_empty() => return Ok(value.trim().to_string()),
            _ if started.elapsed() > timeout => {
                return Err(format!(
                    "companion server did not write ready file {}",
                    path.display()
                ));
            }
            _ => thread::sleep(Duration::from_millis(50)),
        }
    }
}

fn endpoint_port(endpoint: &str) -> Result<u16, String> {
    endpoint
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse().ok())
        .ok_or_else(|| format!("failed to parse companion endpoint port from {endpoint}"))
}

fn ready_file_path(source_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "foreman-companion-{}-{}.ready",
        source_id,
        std::process::id()
    ))
}

fn generate_token() -> String {
    let mut bytes = [0_u8; 24];
    if let Ok(mut file) = fs::File::open("/dev/urandom") {
        use std::io::Read;
        if file.read_exact(&mut bytes).is_ok() {
            return bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        }
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("foreman-{}-{nanos}", std::process::id())
}

fn validate_loopback(host: &str) -> Result<(), String> {
    match host {
        "127.0.0.1" | "localhost" => Ok(()),
        other => Err(format!(
            "connect-ssh requires a loopback remote bind host by default; got {other}"
        )),
    }
}

fn emit_ready(
    config: &ConnectSshConfig,
    local_endpoint: &str,
    remote_endpoint: &str,
    remote_configured: bool,
) {
    if config.json {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "action": "companion.connect-ssh.ready",
                "sourceId": config.source_id,
                "label": config.label,
                "remoteHostLabel": config.label,
                "localEndpoint": local_endpoint,
                "remoteEndpoint": remote_endpoint,
                "remoteConfigured": remote_configured,
                "allowSend": config.allow_send,
            })
        );
    } else {
        eprintln!(
            "Foreman companion connected to remote SSH host {} as source {} ({})",
            config.host, config.source_id, config.label
        );
        eprintln!("Remote endpoint: {remote_endpoint}");
    }
}

struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    fn child_mut(&mut self) -> &mut Child {
        self.child.as_mut().expect("child guard should own child")
    }

    fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = terminate(&mut child);
        }
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn terminate(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|error| error.to_string())?
        .is_none()
    {
        child.kill().map_err(|error| error.to_string())?;
        let _ = child.wait();
    }
    Ok(())
}

fn validate_connect_ssh_config(config: &ConnectSshConfig) -> Result<(), String> {
    validate_loopback(&config.remote_bind_host)?;
    validate_ssh_value("host", &config.host)?;
    validate_ssh_value("ssh", &config.ssh)?;
    validate_ssh_value("remote_foreman", &config.remote_foreman)?;
    for arg in &config.extra_ssh_args {
        validate_ssh_value("extra_ssh_args", arg)?;
    }
    Ok(())
}

fn validate_ssh_value(field: &str, value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("connect-ssh {field} must not be empty"));
    }
    if value.starts_with('-') {
        return Err(format!("connect-ssh {field} must not start with '-'"));
    }
    if value.contains('\0') || value.contains('\n') || value.contains('\r') {
        return Err(format!(
            "connect-ssh {field} must not contain control line breaks"
        ));
    }
    Ok(())
}

pub fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn shell_quote_os(value: &OsStr) -> String {
    shell_quote(&value.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ConnectSshConfig {
        ConnectSshConfig {
            host: "remote-dev".to_string(),
            source_id: "workstation".to_string(),
            label: "Workstation".to_string(),
            remote_port: 4040,
            remote_bind_host: "127.0.0.1".to_string(),
            local_bind: "127.0.0.1:0".to_string(),
            allow_send: true,
            token: Some("tok en".to_string()),
            activation_command: None,
            activation_timeout_ms: 2_000,
            remote_foreman: "foreman".to_string(),
            remote_config_file: Some(PathBuf::from("/tmp/foreman config.toml")),
            ssh: "ssh".to_string(),
            extra_ssh_args: Vec::new(),
            replace: false,
            no_remote_config: false,
            json: true,
            config_file: None,
            tmux_socket: None,
            tmux_server_name: None,
        }
    }

    #[test]
    fn shell_quote_handles_spaces_and_quotes() {
        assert_eq!(shell_quote("abc def"), "'abc def'");
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
    }

    #[test]
    fn remote_foreman_prefix_includes_config_file() {
        assert_eq!(
            remote_foreman_prefix(&config()),
            "'foreman' --config-file '/tmp/foreman config.toml'"
        );
    }

    #[test]
    fn validates_loopback_remote_bind() {
        assert!(validate_loopback("127.0.0.1").is_ok());
        assert!(validate_loopback("0.0.0.0").is_err());
    }
}
