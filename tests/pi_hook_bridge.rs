use foreman::config::default_pi_native_dir;
use std::process::Command;
use tempfile::tempdir;

fn hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-pi-hook")
}

#[test]
fn hook_binary_uses_tmux_pane_env_for_agent_end_events() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let native_dir = temp_dir.path().join("native");

    let output = Command::new(hook_bin())
        .args(["--native-dir"])
        .arg(&native_dir)
        .args(["--event", "agent-end"])
        .env("TMUX_PANE", "%8")
        .output()
        .expect("hook command should run");

    assert!(output.status.success(), "{output:?}");
    let contents =
        std::fs::read_to_string(native_dir.join("%8.json")).expect("signal should exist");
    assert_eq!(contents, r#"{"status":"idle","activity_score":40}"#);
}

#[test]
fn hook_binary_resolves_default_native_dir_from_foreman_state_paths() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let config_root = temp_dir.path().join("config-home");
    let log_dir = temp_dir.path().join("state").join("logs");
    std::fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    std::fs::create_dir_all(&log_dir).expect("log dir should exist");

    let output = Command::new(hook_bin())
        .args(["--event", "agent-start"])
        .env("TMUX_PANE", "%5")
        .env("FOREMAN_CONFIG_HOME", &config_root)
        .env("FOREMAN_LOG_DIR", &log_dir)
        .output()
        .expect("hook command should run");

    assert!(output.status.success(), "{output:?}");
    let native_dir = default_pi_native_dir(&log_dir);
    let contents =
        std::fs::read_to_string(native_dir.join("%5.json")).expect("signal should exist");
    assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
}

#[test]
fn hook_binary_uses_configured_native_dir_when_present() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let config_root = temp_dir.path().join("config-home");
    let configured_native_dir = temp_dir.path().join("custom-native");
    std::fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    std::fs::write(
        config_root.join("foreman/config.toml"),
        format!(
            r#"
[integrations.pi]
native_dir = "{}"
"#,
            configured_native_dir.display()
        ),
    )
    .expect("config should exist");

    let output = Command::new(hook_bin())
        .args(["--event", "session-shutdown"])
        .env("TMUX_PANE", "%3")
        .env("FOREMAN_CONFIG_HOME", &config_root)
        .output()
        .expect("hook command should run");

    assert!(output.status.success(), "{output:?}");
    let contents = std::fs::read_to_string(configured_native_dir.join("%3.json"))
        .expect("signal should exist");
    assert_eq!(contents, r#"{"status":"idle","activity_score":40}"#);
}
