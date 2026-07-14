use serde_json::Value;
use std::process::{Command, Output, Stdio};

fn foreman_command(root: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_foreman"));
    command
        .arg("--config-file")
        .arg(root.join("config.toml"))
        .arg("--log-dir")
        .arg(root.join("state/logs"));
    command
}

fn register(root: &std::path::Path, source_id: &str, terminal_uuid: &str) -> Output {
    foreman_command(root)
        .args([
            "sources",
            "display",
            "register",
            source_id,
            "--provider",
            "ghostty",
            "--terminal-uuid",
            terminal_uuid,
            "--json",
        ])
        .output()
        .expect("run display registration")
}

#[test]
fn concurrent_process_registrations_preserve_both_sources() {
    let root = tempfile::tempdir().expect("temp dir");
    let mut first = foreman_command(root.path());
    first
        .args([
            "sources",
            "display",
            "register",
            "remote-a",
            "--terminal-uuid",
            "terminal-a",
            "--json",
        ])
        .stdout(Stdio::piped());
    let mut second = foreman_command(root.path());
    second
        .args([
            "sources",
            "display",
            "register",
            "remote-b",
            "--terminal-uuid",
            "terminal-b",
            "--json",
        ])
        .stdout(Stdio::piped());

    let first = first.spawn().expect("spawn first registration");
    let second = second.spawn().expect("spawn second registration");
    assert!(first
        .wait_with_output()
        .expect("first output")
        .status
        .success());
    assert!(second
        .wait_with_output()
        .expect("second output")
        .status
        .success());

    let output = foreman_command(root.path())
        .args(["sources", "display", "list", "--json"])
        .output()
        .expect("list registrations");
    assert!(output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("list JSON");
    let registrations = value["registrations"].as_array().expect("registrations");
    assert_eq!(registrations.len(), 2);
    assert!(registrations
        .iter()
        .any(|entry| entry["sourceId"] == "remote-a"));
    assert!(registrations
        .iter()
        .any(|entry| entry["sourceId"] == "remote-b"));
}

#[test]
fn ownership_mismatch_returns_json_and_nonzero_exit() {
    let root = tempfile::tempdir().expect("temp dir");
    let registered = register(root.path(), "remote-dev", "terminal-exact");
    assert!(registered.status.success());

    let output = foreman_command(root.path())
        .args([
            "sources",
            "display",
            "unregister",
            "remote-dev",
            "--handle",
            "stale-owner",
            "--json",
        ])
        .output()
        .expect("unregister with stale owner");

    assert!(!output.status.success());
    let value: Value = serde_json::from_slice(&output.stdout).expect("failure JSON");
    assert_eq!(value["ok"], false);
    assert_eq!(value["code"], "source.display.ownership-mismatch");
}
