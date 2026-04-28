mod support;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-claude-hook")
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("script should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .expect("script metadata should exist")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("script should be executable");
    }
}

fn wait_for_file_contents(path: &Path, needle: &str) {
    for _ in 0..400 {
        if let Ok(contents) = fs::read_to_string(path) {
            if contents.contains(needle) {
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!(
        "file {} never contained expected text: {}",
        path.display(),
        needle
    );
}

fn wait_for_notification_or_diagnose(
    notification_path: &Path,
    hook_trace_path: &Path,
    payload_trace_path: &Path,
    log_path: &Path,
    signal_path: &Path,
    needle: &str,
) {
    for _ in 0..400 {
        if let Ok(contents) = fs::read_to_string(notification_path) {
            if contents.contains(needle) {
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    let notification_contents =
        fs::read_to_string(notification_path).unwrap_or_else(|_| "<missing>".to_string());
    let hook_trace =
        fs::read_to_string(hook_trace_path).unwrap_or_else(|_| "<missing>".to_string());
    let payload_trace =
        fs::read_to_string(payload_trace_path).unwrap_or_else(|_| "<missing>".to_string());
    let log_contents = fs::read_to_string(log_path).unwrap_or_else(|_| "<missing>".to_string());
    let signal_contents =
        fs::read_to_string(signal_path).unwrap_or_else(|_| "<missing>".to_string());

    panic!(
        "file {} never contained expected text: {}\nnotification={}\nhook_trace={}\npayloads=\n{}\nsignal={}\nlogs=\n{}",
        notification_path.display(),
        needle,
        notification_contents,
        hook_trace,
        payload_trace,
        signal_contents,
        log_contents
    );
}

fn claude_is_available() -> bool {
    Command::new("claude")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

#[test]
#[ignore = "requires Claude Code auth, network, and explicit opt-in"]
fn real_claude_prompt_sent_through_dashboard_emits_completion_notification() {
    if std::env::var_os("FOREMAN_REAL_CLAUDE_E2E").is_none() {
        eprintln!("set FOREMAN_REAL_CLAUDE_E2E=1 to run this test");
        return;
    }

    if !claude_is_available() {
        eprintln!("claude CLI is not available in PATH");
        return;
    }

    let fixture = TmuxFixture::new();
    let temp_dir = tempdir().expect("temp dir should exist");
    let config_root = temp_dir.path().join("config");
    let log_dir = temp_dir.path().join("logs");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");

    let hook_trace = temp_dir.path().join("hook-trace.log");
    let payload_trace = temp_dir.path().join("hook-payloads.log");
    let notification_file = temp_dir.path().join("notification.log");
    let hook_wrapper = bin_dir.join("claude-hook-wrapper.sh");
    write_executable(
        &hook_wrapper,
        "#!/bin/sh\nset -eu\nEVENT=\"$1\"\nHOOK_BIN=\"$2\"\nNATIVE_DIR=\"$3\"\nTRACE_FILE=\"$4\"\nPAYLOAD_TRACE=\"$5\"\nTMP_INPUT=$(mktemp)\ncat > \"$TMP_INPUT\"\nprintf '%s\\n' \"$EVENT\" >> \"$TRACE_FILE\"\nprintf '=== %s ===\\n' \"$EVENT\" >> \"$PAYLOAD_TRACE\"\ncat \"$TMP_INPUT\" >> \"$PAYLOAD_TRACE\"\nprintf '\\n' >> \"$PAYLOAD_TRACE\"\nif [ \"$EVENT\" = \"stop\" ]; then\n  sleep 2\nfi\n\"$HOOK_BIN\" --native-dir \"$NATIVE_DIR\" < \"$TMP_INPUT\"\nSTATUS=$?\nrm -f \"$TMP_INPUT\"\nexit \"$STATUS\"\n",
    );
    write_executable(
        &bin_dir.join("notify-send"),
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"{}\"\n",
            notification_file.display()
        ),
    );

    let settings_path = temp_dir.path().join("claude-settings.json");
    fs::write(
        &settings_path,
        format!(
            r#"{{
  "hooks": {{
    "UserPromptSubmit": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} submit {hook_bin} {native_dir} {hook_trace} {payload_trace}"
          }}
        ]
      }}
    ],
    "Stop": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} stop {hook_bin} {native_dir} {hook_trace} {payload_trace}"
          }}
        ]
      }}
    ],
    "StopFailure": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} stop_failure {hook_bin} {native_dir} {hook_trace} {payload_trace}"
          }}
        ]
      }}
    ],
    "Notification": [
      {{
        "matcher": "permission_prompt|elicitation_dialog",
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} notification {hook_bin} {native_dir} {hook_trace} {payload_trace}"
          }}
        ]
      }}
    ]
  }}
}}"#,
            hook_wrapper = hook_wrapper.display(),
            hook_bin = hook_bin(),
            native_dir = native_dir.display(),
            hook_trace = hook_trace.display(),
            payload_trace = payload_trace.display(),
        ),
    )
    .expect("settings should be written");

    fs::write(
        config_root.join("foreman/config.toml"),
        format!(
            r#"
[notifications]
enabled = true
cooldown_ticks = 1
backends = ["notify-send"]
active_profile = "completion-only"

[integrations.claude_code]
mode = "native"
native_dir = "{native_dir}"
"#,
            native_dir = native_dir.display(),
        ),
    )
    .expect("config should be written");

    let agent_script = format!(
        "import os\nimport subprocess\nimport sys\nos.environ['PATH'] = {bin_dir:?} + os.pathsep + os.environ.get('PATH', '')\nsettings_path = {settings_path:?}\nprint('Claude hook loop ready', flush=True)\nfor line in sys.stdin:\n    prompt = line.rstrip('\\n')\n    print(f'PROMPT:{{prompt}}', flush=True)\n    subprocess.run(['claude', '-p', '--settings', settings_path, prompt], check=True)\n    print('__CLAUDE_DONE__', flush=True)\n",
        bin_dir = bin_dir.display().to_string(),
        settings_path = settings_path.display().to_string(),
    );
    let agent_command = format!("python3 -u -c {}", shell_quote(&agent_script));
    let agent_pane = fixture.new_session("alpha", &agent_command);
    fixture.wait_for_capture(&agent_pane, "Claude hook loop ready");

    let helper_pane = fixture.new_session("beta", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&helper_pane, "Claude Code ready");

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 40",
        bin_dir.display(),
        config_root.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "Foreman", 200);
    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "Foreman | NORMAL", 200);
    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "alpha", 200);
    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "beta", 200);

    fixture.send_keys(&dashboard_pane, &["j", "j"]);
    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "Compose ->", 80);

    fixture.send_keys(
        &dashboard_pane,
        &[
            "i", "O", "n", "l", "y", "Space", "o", "u", "t", "p", "u", "t", "Space", "O", "K",
            "C-s",
        ],
    );
    fixture.wait_for_capture_attempts(&agent_pane, "PROMPT:Only output OK", 120);
    fixture.send_keys(&dashboard_pane, &["j", "j", "j"]);
    fixture.wait_for_alt_capture_attempts(&dashboard_pane, "beta / foreman / foreman", 80);

    fixture.wait_for_capture_attempts(&agent_pane, "__CLAUDE_DONE__", 400);
    wait_for_file_contents(&hook_trace, "submit");
    wait_for_file_contents(&hook_trace, "stop");
    wait_for_notification_or_diagnose(
        &notification_file,
        &hook_trace,
        &payload_trace,
        &log_dir.join("latest.log"),
        &native_dir.join(format!("{agent_pane}.json")),
        "completion|Foreman: agent ready",
    );

    let signal_contents = fs::read_to_string(native_dir.join(format!("{agent_pane}.json")))
        .expect("native signal should exist");
    assert!(signal_contents.contains(r#""status":"idle""#));

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture_attempts(&dashboard_pane, "FOREMAN_EXITED", 120);

    let log_contents =
        fs::read_to_string(log_dir.join("latest.log")).expect("latest log should be readable");
    assert!(log_contents.contains("notification_decision"));
    assert!(log_contents.contains("reason=working_became_ready"));
    assert!(log_contents.contains("notification_backend_selected"));
}
