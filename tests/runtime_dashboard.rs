mod support;

use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn claude_hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-claude-hook")
}

fn sleeping_shell_command(workdir: &Path, banner: &str) -> String {
    let script = format!(
        "cd {} && printf '%s\\n' {} && exec sleep 600",
        shell_escape(workdir.display().to_string().as_str()),
        shell_escape(banner)
    );
    format!("sh -lc {}", shell_escape(&script))
}

fn shell_escape(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

fn send_claude_hook_event(native_dir: &Path, pane_id: &str, payload: &str) {
    let mut child = Command::new(claude_hook_bin())
        .args(["--native-dir", native_dir.to_str().expect("utf-8 path")])
        .env("TMUX_PANE", pane_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("claude hook command should spawn");

    child
        .stdin
        .take()
        .expect("hook stdin should exist")
        .write_all(payload.as_bytes())
        .expect("hook payload should be written");

    let output = child
        .wait_with_output()
        .expect("hook command should finish");
    if !output.status.success() {
        panic!(
            "claude hook failed\nstderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn send_text(fixture: &TmuxFixture, target: &str, text: &str) {
    for ch in text.chars() {
        let key = match ch {
            ' ' => "Space".to_string(),
            _ => ch.to_string(),
        };
        fixture.send_keys(target, &[key.as_str()]);
    }
}

fn wait_for_alt_capture_not_contains(
    fixture: &TmuxFixture,
    target: &str,
    needle: &str,
    attempts: usize,
) {
    for _ in 0..attempts {
        let capture = fixture.capture_alt(target);
        if !capture.contains(needle) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!("pane {target} still contained unexpected text: {needle}");
}

fn wait_for_log_contains(log_path: &Path, needle: &str, attempts: usize) {
    for _ in 0..attempts {
        let contents = fs::read_to_string(log_path).unwrap_or_default();
        if contents.contains(needle) {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!("expected log {} to contain {needle}", log_path.display());
}

fn real_tmux_path() -> String {
    let output = Command::new("sh")
        .args(["-lc", "command -v tmux"])
        .output()
        .expect("command -v tmux should run");
    assert!(
        output.status.success(),
        "command -v tmux failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("tmux path should be utf-8")
        .trim()
        .to_string()
}

fn write_slow_tmux_proxy(dir: &Path, delay_seconds: &str) {
    let tmux_path = real_tmux_path();
    let script = format!(
        "#!/bin/sh\nsleep {delay_seconds}\nexec {} \"$@\"\n",
        shell_escape(&tmux_path)
    );
    let path = dir.join("tmux");
    fs::write(&path, script).expect("slow tmux proxy should be written");
    let mut permissions = fs::metadata(&path)
        .expect("slow tmux proxy metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("slow tmux proxy should be executable");
}

#[test]
fn interactive_binary_renders_dashboard_and_sends_input_to_selected_agent() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "INPUT:"),
    );
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let latest_log_path = log_dir.path().join("latest.log");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify --debug",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 180, 48);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");

    fixture.send_keys(&dashboard_pane, &["j", "j"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Compose ->");
    fixture.send_keys(&dashboard_pane, &["o"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "attention->recent");

    fixture.send_keys(&dashboard_pane, &["i", "h", "i", "Enter"]);
    fixture.wait_for_capture(&agent_pane, "INPUT:hi");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
    wait_for_log_contains(&latest_log_path, "ui_preferences_write", 40);
}

#[test]
fn interactive_binary_uses_cached_inventory_before_slow_live_refresh() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "CLAUDE:"),
    );
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let latest_log_path = log_dir.path().join("latest.log");

    let warm_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --popup --debug --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let warm_pane = fixture.new_session(
        "dashboard-warm",
        &fixture.keep_alive_command(&warm_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&warm_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&warm_pane, "alpha");
    wait_for_log_contains(
        &latest_log_path,
        "timing operation=startup_cache_write outcome=written",
        120,
    );
    fixture.send_keys(&warm_pane, &["q"]);
    fixture.wait_for_capture(&warm_pane, "FOREMAN_EXITED");

    let slow_tmux_dir = tempdir().expect("slow tmux dir should exist");
    write_slow_tmux_proxy(slow_tmux_dir.path(), "2.0");
    let cached_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --popup --debug --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        slow_tmux_dir.path().display(),
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let cached_pane = fixture.new_session(
        "dashboard-cache",
        &fixture.keep_alive_command(&cached_command, "FOREMAN_EXITED"),
    );

    let cached_started = Instant::now();
    fixture.wait_for_alt_capture(&cached_pane, "cached ");
    fixture.wait_for_alt_capture(&cached_pane, "alpha");
    let cached_visible_at = cached_started.elapsed();
    wait_for_alt_capture_not_contains(&fixture, &cached_pane, "cached ", 120);
    let live_refresh_replaced_at = cached_started.elapsed();
    assert!(
        cached_visible_at + Duration::from_millis(400) < live_refresh_replaced_at,
        "expected cached startup frame to land well before live refresh replaced it (cached_visible_at={cached_visible_at:?}, live_refresh_replaced_at={live_refresh_replaced_at:?})"
    );

    fixture.send_keys(&cached_pane, &["q"]);
    fixture.wait_for_capture(&cached_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_help_and_harness_filter_walkthrough_stays_actionable() {
    let fixture = TmuxFixture::new();
    let claude_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "CLAUDE:"),
    );
    let codex_pane = fixture.new_session(
        "beta",
        &fixture.interactive_echo_command("Codex CLI waiting for your input", "CODEX:"),
    );
    fixture.wait_for_capture(&claude_pane, "Claude Code ready");
    fixture.wait_for_capture(&codex_pane, "Codex CLI waiting for your input");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 180, 48);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "alpha");
    fixture.wait_for_alt_capture(&dashboard_pane, "beta");
    fixture.wait_for_alt_capture(&dashboard_pane, "Keys • Sidebar");
    fixture.wait_for_alt_capture(&dashboard_pane, "Status source:  compatibility heuristic");

    fixture.send_keys(&dashboard_pane, &["?"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Focus: Sidebar");
    fixture.wait_for_alt_capture(&dashboard_pane, "Legend");
    fixture.wait_for_alt_capture(&dashboard_pane, "Claude");
    fixture.wait_for_alt_capture(&dashboard_pane, "Target source: compatibility heuristic");
    fixture.send_keys(&dashboard_pane, &["Escape"]);
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "Legend", 40);

    fixture.send_keys(&dashboard_pane, &["h"]);
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "beta", 40);
    fixture.send_keys(&dashboard_pane, &["h"]);
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "alpha", 40);
    fixture.send_keys(&dashboard_pane, &["h"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "alpha");
    fixture.wait_for_alt_capture(&dashboard_pane, "beta");

    fixture.send_keys(&dashboard_pane, &["i", "o", "k", "Enter"]);
    fixture.wait_for_capture(&codex_pane, "CODEX:ok");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_footer_tracks_focus_and_help_explains_provenance() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "CLAUDE:"),
    );
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 160, 40);

    fixture.wait_for_alt_capture(&dashboard_pane, "Keys • Sidebar");
    fixture.wait_for_alt_capture(&dashboard_pane, "Sidebar: j/k move");
    fixture.wait_for_alt_capture(&dashboard_pane, "Status source:  compatibility heuristic");

    fixture.send_keys(&dashboard_pane, &["2"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Keys • Details");
    fixture.wait_for_alt_capture(&dashboard_pane, "Details: j/k scroll");

    fixture.send_keys(&dashboard_pane, &["3"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Keys • Compose");
    fixture.wait_for_alt_capture(&dashboard_pane, "Compose: Enter or i start");

    fixture.send_keys(&dashboard_pane, &["?"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Focus: Compose");
    fixture.wait_for_alt_capture(&dashboard_pane, "Target source: compatibility heuristic");
    fixture.wait_for_alt_capture(
        &dashboard_pane,
        "compatibility heuristic = tmux-observed status",
    );

    fixture.send_keys(&dashboard_pane, &["Escape"]);
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "Focus: Compose", 40);
    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_help_scrolls_in_small_layout() {
    let fixture = TmuxFixture::new();
    let claude_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "CLAUDE:"),
    );
    fixture.wait_for_capture(&claude_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 88, 20);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.send_keys(&dashboard_pane, &["?"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Legend");
    fixture.wait_for_alt_capture(&dashboard_pane, "Scroll j/k");
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "h cycles visible harnesses", 40);

    fixture.send_keys(
        &dashboard_pane,
        &[
            "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j",
            "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j", "j",
        ],
    );
    fixture.wait_for_alt_capture(&dashboard_pane, "h cycles visible harnesses");

    fixture.send_keys(
        &dashboard_pane,
        &[
            "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k",
            "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k", "k",
        ],
    );
    fixture.wait_for_alt_capture(&dashboard_pane, "Legend");

    fixture.send_keys(&dashboard_pane, &["Escape"]);
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "Scroll j/k", 40);
    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_popup_keeps_side_by_side_tree_layout() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "CLAUDE:"),
    );
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --popup --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 88, 20);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "* Targets");
    fixture.wait_for_alt_capture(&dashboard_pane, "Details");

    let capture = fixture.capture_alt(&dashboard_pane);
    assert!(
        capture
            .lines()
            .any(|line| line.contains("* Targets") && line.contains("Details")),
        "expected popup layout to keep sidebar and details side-by-side:\n{capture}"
    );

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_popup_focus_action_exits_after_success() {
    let fixture = TmuxFixture::new();
    let helper_pane = fixture.new_session("alpha", &fixture.shell_command("plain shell"));
    let agent_pane =
        fixture.split_window(&helper_pane, &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify --popup",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ alpha");
    fixture.wait_for_alt_capture(&dashboard_pane, "• ✦ foreman");

    fixture.send_keys(&dashboard_pane, &["f"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
    assert_eq!(fixture.active_pane_in("alpha"), agent_pane);
}

#[test]
fn interactive_binary_renders_loading_before_first_inventory_fill() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let fake_bin = tempdir().expect("fake tmux dir should exist");
    write_slow_tmux_proxy(fake_bin.path(), "0.25");
    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 200 --no-notify",
        fake_bin.path().display(),
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Loading tmux inventory");
    fixture.wait_for_alt_capture(&dashboard_pane, "alpha");
    wait_for_alt_capture_not_contains(&fixture, &dashboard_pane, "Loading tmux inventory", 60);

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_surfaces_claude_native_status_and_attention_view() {
    let fixture = TmuxFixture::new();
    let workspaces = tempdir().expect("workspace root should exist");
    let alpha_dir = workspaces.path().join("alpha");
    let beta_dir = workspaces.path().join("beta");
    std::fs::create_dir_all(&alpha_dir).expect("alpha dir should exist");
    std::fs::create_dir_all(&beta_dir).expect("beta dir should exist");

    let alpha_pane = fixture.new_session(
        "alpha",
        &sleeping_shell_command(&alpha_dir, "Claude Code ready"),
    );
    let beta_pane = fixture.new_session(
        "beta",
        &sleeping_shell_command(&beta_dir, "Codex CLI waiting for your input"),
    );
    fixture.wait_for_capture(&alpha_pane, "Claude Code ready");
    fixture.wait_for_capture(&beta_pane, "Codex CLI waiting for your input");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let native_dir = tempdir().expect("native dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify --claude-native-dir {}",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display(),
        native_dir.path().display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 180, 48);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "Status source:  compatibility heuristic");
    fixture.send_keys(&dashboard_pane, &["/"]);
    send_text(&fixture, &dashboard_pane, "alpha");
    fixture.send_keys(&dashboard_pane, &["Enter"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Target:         ✦ alpha");
    fixture.send_keys(&dashboard_pane, &["o"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Sort attention->recent");

    send_claude_hook_event(
        native_dir.path(),
        &alpha_pane,
        r#"{"hook_event_name":"UserPromptSubmit","prompt":"manual smoke"}"#,
    );
    fixture.wait_for_alt_capture(&dashboard_pane, "Status source:  native hook");

    send_claude_hook_event(
        native_dir.path(),
        &alpha_pane,
        r#"{"hook_event_name":"Notification","notification_type":"permission_prompt"}"#,
    );
    fixture.wait_for_alt_capture(&dashboard_pane, "Status:         ATTENTION");
    fixture.wait_for_alt_capture(&dashboard_pane, "Status source:  native hook");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
fn interactive_binary_popup_focus_action_switches_cross_session_target() {
    let fixture = TmuxFixture::new();
    let workspaces = tempdir().expect("workspace root should exist");
    let alpha_dir = workspaces.path().join("alpha");
    let beta_dir = workspaces.path().join("beta");
    std::fs::create_dir_all(&alpha_dir).expect("alpha dir should exist");
    std::fs::create_dir_all(&beta_dir).expect("beta dir should exist");

    let alpha_helper =
        fixture.new_session("alpha", &sleeping_shell_command(&alpha_dir, "plain shell"));
    let _alpha_agent = fixture.split_window(
        &alpha_helper,
        &sleeping_shell_command(&alpha_dir, "Claude Code ready"),
    );
    let beta_helper = fixture.new_session("beta", &sleeping_shell_command(&beta_dir, "beta shell"));
    let beta_agent = fixture.split_window(
        &beta_helper,
        &sleeping_shell_command(&beta_dir, "Codex CLI waiting for your input"),
    );
    fixture.wait_for_capture(&beta_agent, "Codex CLI waiting for your input");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify --popup",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );
    fixture.resize_window("dashboard", 180, 48);

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ beta");
    fixture.send_keys(&dashboard_pane, &["/"]);
    send_text(&fixture, &dashboard_pane, "beta");
    fixture.send_keys(&dashboard_pane, &["Enter"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Target:         ◎ beta");
    fixture.wait_for_alt_capture(&dashboard_pane, beta_dir.display().to_string().as_str());

    fixture.send_keys(&dashboard_pane, &["f"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
    assert_eq!(fixture.active_pane_in("beta"), beta_agent);
}

#[test]
fn interactive_binary_spawn_modal_submits_with_enter() {
    let fixture = TmuxFixture::new();
    let agent_pane = fixture.new_session("alpha", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&agent_pane, "Claude Code ready");

    let config_dir = tempdir().expect("config dir should exist");
    let log_dir = tempdir().expect("log dir should exist");
    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --no-notify",
        config_dir.path().display(),
        log_dir.path().display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ alpha");
    fixture.send_keys(
        &dashboard_pane,
        &["N", "s", "l", "e", "e", "p", "Space", "6", "0", "Enter"],
    );

    let adapter = TmuxAdapter::new(SystemTmuxBackend::new(Some(
        fixture.socket_path().to_path_buf(),
    )));
    let mut window_count = 0;
    for _ in 0..20 {
        let inventory = adapter.load_inventory(20).expect("inventory should load");
        let session = inventory
            .session(&"alpha".into())
            .or_else(|| {
                inventory
                    .sessions
                    .iter()
                    .find(|session| session.name == "alpha")
            })
            .expect("alpha session should exist");
        window_count = session.windows.len();
        if window_count == 2 {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(window_count, 2);

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}
