mod support;

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use support::tmux::TmuxFixture;
use tempfile::TempDir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn shell_escape(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

fn fake_pi_command(workdir: &Path, title: &str) -> String {
    let script = format!(
        "cd {} && printf '\\033]2;%s\\007' {} && exec sleep 600",
        shell_escape(workdir.display().to_string().as_str()),
        shell_escape(title)
    );
    format!("sh -lc {}", shell_escape(&script))
}

fn pi_subagent_async_dir() -> PathBuf {
    std::env::temp_dir()
        .join(format!("pi-subagents-uid-{}", current_uid()))
        .join("async-subagent-runs")
}

fn current_uid() -> u32 {
    let output = Command::new("id")
        .arg("-u")
        .output()
        .expect("id -u should run");
    assert!(output.status.success(), "id -u failed");
    String::from_utf8(output.stdout)
        .expect("uid should be utf-8")
        .trim()
        .parse()
        .expect("uid should be numeric")
}

fn write_subagent_status(root: &Path, run_id: &str, cwd: &Path, current_tool: &str) -> PathBuf {
    write_subagent_status_with_activity(root, run_id, cwd, current_tool, "active")
}

fn write_attention_subagent_status(
    root: &Path,
    run_id: &str,
    cwd: &Path,
    current_tool: &str,
) -> PathBuf {
    write_subagent_status_with_activity(root, run_id, cwd, current_tool, "needs_attention")
}

fn write_subagent_status_with_activity(
    root: &Path,
    run_id: &str,
    cwd: &Path,
    current_tool: &str,
    activity_state: &str,
) -> PathBuf {
    let cwd = cwd
        .canonicalize()
        .expect("subagent cwd should canonicalize like tmux pane paths");
    let run_dir = root.join(run_id);
    fs::create_dir_all(&run_dir).expect("run dir should be created");
    let status_path = run_dir.join("status.json");
    fs::write(
        &status_path,
        serde_json::to_string_pretty(&json!({
            "runId": run_id,
            "state": "running",
            "activityState": activity_state,
            "currentTool": current_tool,
            "currentPath": cwd.join("src/lib.rs"),
            "cwd": cwd,
            "turnCount": 1,
            "toolCount": 1,
        }))
        .expect("status json should serialize"),
    )
    .expect("status should be written");
    status_path
}

fn run_foreman_json(args: &[&str]) -> Value {
    let output = Command::new(foreman_bin())
        .args(args)
        .output()
        .expect("foreman should run");
    assert!(
        output.status.success(),
        "foreman {:?} failed\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("foreman stdout should be json")
}

fn entry_for_pane<'a>(payload: &'a Value, pane_id: &str) -> &'a Value {
    payload["entries"]
        .as_array()
        .expect("entries should be an array")
        .iter()
        .find(|entry| entry["paneId"] == pane_id)
        .unwrap_or_else(|| panic!("missing pane {pane_id} in {payload:#}"))
}

fn pi_subagents_card(payload: &Value) -> &Value {
    payload["extensionCards"]
        .as_array()
        .expect("extensionCards should be an array")
        .iter()
        .find(|card| card["id"] == "pi-subagents")
        .unwrap_or_else(|| panic!("missing pi-subagents card in {payload:#}"))
}

#[test]
fn agent_sim_pi_subagent_activity_promotes_parent_without_leaking_to_nested_pane() {
    let fixture = TmuxFixture::new();
    let temp = tempfile::tempdir().expect("temp dir should exist");
    let repo = temp.path().join("repo");
    let module = repo.join("module");
    fs::create_dir_all(&module).expect("workspace dirs should exist");

    let parent_pane =
        fixture.new_session("pi-parent", &fake_pi_command(&repo, "π - agent-sim parent"));
    let module_pane = fixture.new_session(
        "pi-module",
        &fake_pi_command(&module, "π - agent-sim module"),
    );
    fixture.wait_for_capture(&parent_pane, "");
    fixture.wait_for_capture(&module_pane, "");

    let async_root = pi_subagent_async_dir();
    let run_prefix = format!(
        "foreman-agent-sim-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be monotonic enough")
            .as_millis()
    );
    let root_status =
        write_subagent_status(&async_root, &format!("{run_prefix}-root"), &repo, "Bash");
    let module_status = write_subagent_status(
        &async_root,
        &format!("{run_prefix}-module"),
        &module,
        "Edit",
    );
    let _cleanup = StatusCleanup::new(vec![root_status, module_status]);

    let socket = fixture.socket_path().to_str().expect("socket path utf-8");
    let agents = run_foreman_json(&["--tmux-socket", socket, "agents", "--json", "--all-panes"]);
    let parent = entry_for_pane(&agents, &parent_pane);
    assert_eq!(parent["harness"], "pi");
    assert_eq!(parent["integrationMode"], "native");
    assert_eq!(parent["status"], "working");
    assert_eq!(parent["activeRunCount"], 2);

    let nested = entry_for_pane(&agents, &module_pane);
    assert_eq!(nested["harness"], "pi");
    assert_eq!(nested["integrationMode"], "native");
    assert_eq!(nested["status"], "working");
    assert_eq!(nested["activeRunCount"], 1);

    let parent_cards = run_foreman_json(&[
        "--tmux-socket",
        socket,
        "extensions",
        "--pane",
        parent_pane.as_str(),
        "--json",
    ]);
    let parent_subagents = pi_subagents_card(&parent_cards);
    assert_eq!(parent_subagents["summary"], "2 running");

    let module_cards = run_foreman_json(&[
        "--tmux-socket",
        socket,
        "extensions",
        "--pane",
        module_pane.as_str(),
        "--json",
    ]);
    let module_subagents = pi_subagents_card(&module_cards);
    assert_eq!(module_subagents["summary"], "1 running");
}

#[test]
fn agent_sim_pi_subagent_attention_promotes_parent_and_card() {
    let fixture = TmuxFixture::new();
    let temp = tempfile::tempdir().expect("temp dir should exist");
    let repo = temp.path().join("repo");
    fs::create_dir_all(&repo).expect("workspace dir should exist");

    let pane = fixture.new_session(
        "pi-attention",
        &fake_pi_command(&repo, "π - agent-sim attention"),
    );
    fixture.wait_for_capture(&pane, "");

    let async_root = pi_subagent_async_dir();
    let run_id = format!(
        "foreman-agent-sim-attention-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should be monotonic enough")
            .as_millis()
    );
    let status = write_attention_subagent_status(&async_root, &run_id, &repo, "Bash");
    let _cleanup = StatusCleanup::new(vec![status]);

    let socket = fixture.socket_path().to_str().expect("socket path utf-8");
    let agents = run_foreman_json(&["--tmux-socket", socket, "agents", "--json", "--all-panes"]);
    let entry = entry_for_pane(&agents, &pane);
    assert_eq!(entry["harness"], "pi");
    assert_eq!(entry["integrationMode"], "native");
    assert_eq!(entry["status"], "needs-attention");
    assert_eq!(entry["activeRunCount"], 1);

    let cards = run_foreman_json(&[
        "--tmux-socket",
        socket,
        "extensions",
        "--pane",
        pane.as_str(),
        "--json",
    ]);
    let subagents = pi_subagents_card(&cards);
    assert_eq!(subagents["status"], "needs-attention");
    assert_eq!(subagents["statusLabel"], "NEEDS ATTENTION");
    assert_eq!(subagents["summary"], "1 running");
}

struct StatusCleanup {
    dirs: Vec<PathBuf>,
    _temp_guard: Option<TempDir>,
}

impl StatusCleanup {
    fn new(status_paths: Vec<PathBuf>) -> Self {
        let dirs = status_paths
            .into_iter()
            .filter_map(|path| path.parent().map(Path::to_path_buf))
            .collect();
        Self {
            dirs,
            _temp_guard: None,
        }
    }
}

impl Drop for StatusCleanup {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }
}
