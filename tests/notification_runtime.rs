mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::thread;
use std::time::Duration;
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).expect("script should be written");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("permissions should update");
}

fn write_atomic(path: &std::path::Path, contents: &str) {
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, contents).expect("temp file should be written");
    fs::rename(&temp_path, path).expect("temp file should replace target");
}

fn wait_for_file_contents(path: &std::path::Path, needle: &str) {
    for _ in 0..120 {
        if let Ok(contents) = fs::read_to_string(path) {
            if contents.contains(needle) {
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!("file {} never contained {}", path.display(), needle);
}

#[test]
fn runtime_uses_configured_notification_backend_order() {
    let fixture = TmuxFixture::new();
    let helper_pane = fixture.new_session("beta", &fixture.shell_command("Claude Code helper"));
    fixture.wait_for_capture(&helper_pane, "Claude Code helper");
    let pane_id = fixture.new_session("alpha", &fixture.shell_command("Claude Code session"));
    fixture.wait_for_capture(&pane_id, "Claude Code session");

    let temp_dir = tempdir().expect("temp dir should exist");
    let config_dir = temp_dir.path().join("config");
    let log_dir = temp_dir.path().join("logs");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir_all(config_dir.join("foreman")).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");

    fs::write(
        config_dir.join("foreman/config.toml"),
        r#"
[notifications]
enabled = true
cooldown_ticks = 1
backends = ["osascript", "notify-send"]
active_profile = "completion-only"
"#,
    )
    .expect("config should be written");

    write_atomic(
        &native_dir.join(format!("{helper_pane}.json")),
        r#"{"status":"working","activity_score":99}"#,
    );

    let notification_file = temp_dir.path().join("notification.txt");
    write_executable(&bin_dir.join("osascript"), "#!/bin/sh\nexit 1\n");
    write_executable(
        &bin_dir.join("notify-send"),
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"{}\"\n",
            notification_file.display()
        ),
    );
    write_executable(
        &bin_dir.join("sh"),
        &format!(
            "#!/bin/sh\nexport PATH=\"{}:$PATH\"\nexec /bin/sh \"$@\"\n",
            bin_dir.display()
        ),
    );

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --claude-native-dir {}",
        bin_dir.display(),
        config_dir.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display(),
        native_dir.display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ beta");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ alpha");
    fixture.send_keys(&dashboard_pane, &["j", "j", "j"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "› ▾ alpha");

    write_atomic(
        &native_dir.join(format!("{helper_pane}.json")),
        r#"{"status":"idle","activity_score":44}"#,
    );

    wait_for_file_contents(&notification_file, "completion|Agent ready:");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");

    let log_contents =
        fs::read_to_string(log_dir.join("latest.log")).expect("latest log should be readable");
    assert!(log_contents.contains("notification_backend_selected"));
    assert!(log_contents.contains("backend=notify-send"));
}

#[test]
fn runtime_honors_attention_only_profile_from_config() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session("alpha", &fixture.shell_command("Claude Code session"));
    fixture.wait_for_capture(&pane_id, "Claude Code session");

    let temp_dir = tempdir().expect("temp dir should exist");
    let config_root = temp_dir.path().join("config");
    let log_dir = temp_dir.path().join("logs");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");

    fs::write(
        config_root.join("foreman/config.toml"),
        r#"
[notifications]
enabled = true
cooldown_ticks = 1
backends = ["notify-send"]
active_profile = "attention-only"
"#,
    )
    .expect("config should be written");

    write_atomic(
        &native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"working","activity_score":99}"#,
    );

    let notification_file = temp_dir.path().join("notification.txt");
    write_executable(
        &bin_dir.join("notify-send"),
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"{}\"\n",
            notification_file.display()
        ),
    );
    write_executable(
        &bin_dir.join("sh"),
        &format!(
            "#!/bin/sh\nexport PATH=\"{}:$PATH\"\nexec /bin/sh \"$@\"\n",
            bin_dir.display()
        ),
    );

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --claude-native-dir {}",
        bin_dir.display(),
        config_root.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display(),
        native_dir.display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");

    write_atomic(
        &native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"idle","activity_score":44}"#,
    );

    wait_for_file_contents(&log_dir.join("latest.log"), "reason=profile_filtered");
    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");

    assert!(!notification_file.exists());

    let log_contents =
        fs::read_to_string(log_dir.join("latest.log")).expect("latest log should be readable");
    assert!(log_contents.contains("notification_decision"));
    assert!(log_contents.contains("reason=profile_filtered"));
}

#[test]
fn interactive_input_then_completion_transition_emits_notification() {
    let fixture = TmuxFixture::new();
    let pane_id = fixture.new_session(
        "alpha",
        &fixture.interactive_echo_command("Claude Code ready", "INPUT:"),
    );
    fixture.wait_for_capture(&pane_id, "Claude Code ready");
    let beta_pane = fixture.new_session("beta", &fixture.shell_command("Claude Code ready"));
    fixture.wait_for_capture(&beta_pane, "Claude Code ready");

    let temp_dir = tempdir().expect("temp dir should exist");
    let config_root = temp_dir.path().join("config");
    let log_dir = temp_dir.path().join("logs");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");

    fs::write(
        config_root.join("foreman/config.toml"),
        r#"
[notifications]
enabled = true
cooldown_ticks = 1
backends = ["notify-send"]
active_profile = "completion-only"
"#,
    )
    .expect("config should be written");

    write_atomic(
        &native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"idle","activity_score":44}"#,
    );

    let notification_file = temp_dir.path().join("notification.txt");
    write_executable(
        &bin_dir.join("notify-send"),
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"{}\"\n",
            notification_file.display()
        ),
    );
    write_executable(
        &bin_dir.join("sh"),
        &format!(
            "#!/bin/sh\nexport PATH=\"{}:$PATH\"\nexec /bin/sh \"$@\"\n",
            bin_dir.display()
        ),
    );

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --claude-native-dir {}",
        bin_dir.display(),
        config_root.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display(),
        native_dir.display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ alpha");
    fixture.wait_for_alt_capture(&dashboard_pane, "▾ beta");

    fixture.send_keys(&dashboard_pane, &["j", "j"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Compose ->");

    fixture.send_keys(&dashboard_pane, &["i", "s", "l", "e", "e", "p", "C-s"]);
    fixture.wait_for_capture(&pane_id, "INPUT:sleep");
    fixture.send_keys(&dashboard_pane, &["j"]);

    write_atomic(
        &native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"working","activity_score":99}"#,
    );
    // Give the runtime enough time to observe the working transition before the
    // ready transition, even under the heavier all-features test sweep.
    thread::sleep(Duration::from_millis(900));
    write_atomic(
        &native_dir.join(format!("{pane_id}.json")),
        r#"{"status":"idle","activity_score":44}"#,
    );

    wait_for_file_contents(&notification_file, "completion|Agent ready:");

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");

    let log_contents =
        fs::read_to_string(log_dir.join("latest.log")).expect("latest log should be readable");
    assert!(log_contents.contains("notification_decision"));
    assert!(log_contents.contains("reason=working_became_ready"));
    assert!(log_contents.contains("notification_backend_selected"));
}
