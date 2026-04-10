mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::thread;
use std::time::Duration;
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn navigation_burst_defers_pull_request_lookup_until_selection_settles() {
    let fixture = TmuxFixture::new();
    let root = tempdir().expect("temp dir should exist");
    let config_dir = root.path().join("config");
    let log_dir = root.path().join("logs");
    let fake_bin = root.path().join("fake-bin");
    let gh_lookup_log = root.path().join("gh-lookups.txt");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&fake_bin).expect("fake bin should exist");

    write_fake_gh(&fake_bin, &gh_lookup_log);

    for (name, banner) in [
        ("alpha", "Claude Code ready"),
        ("beta", "Codex CLI waiting for your input"),
        ("gamma", "Pi ready for the next task"),
    ] {
        let repo = root.path().join(name);
        fs::create_dir_all(&repo).expect("repo dir should exist");
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repo)
            .status()
            .expect("git init should run");

        let pane_id = fixture.new_session(name, &repo_shell_command(&repo, banner));
        fixture.wait_for_capture(&pane_id, banner);
    }

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --debug --tmux-socket {} --poll-interval-ms 500 --capture-lines 20 --no-notify",
        fake_bin.display(),
        config_dir.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");
    fixture.send_keys(&dashboard_pane, &["j", "j", "j", "j", "j", "j"]);

    wait_for_lookup_count(&gh_lookup_log, 2, 60);
    let latest_log_path = log_dir.join("latest.log");
    wait_for_log_contains(&latest_log_path, "trigger=selection-idle", 60);

    let lookups = read_nonempty_lines(&gh_lookup_log);
    assert_eq!(
        lookups.len(),
        2,
        "expected bootstrap and settled-selection lookups"
    );
    assert!(
        lookups[1].ends_with("/gamma"),
        "expected settled lookup to target gamma repo, got {lookups:?}"
    );

    let latest_log = fs::read_to_string(&latest_log_path).expect("latest log should be readable");
    assert!(
        latest_log.contains("timing operation=action action=move-selection"),
        "latest log did not include move-selection timing:\n{latest_log}"
    );
    assert!(
        latest_log.contains("timing operation=pull_request_lookup"),
        "latest log did not include pull-request timing:\n{latest_log}"
    );
    assert!(
        latest_log.contains("trigger=selection-idle"),
        "latest log did not include selection-idle lookup timing:\n{latest_log}"
    );
    assert!(
        latest_log.contains("slow_operation operation=pull_request_lookup"),
        "latest log did not include slow pull-request timing:\n{latest_log}"
    );

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

fn write_fake_gh(fake_bin: &Path, lookup_log: &Path) {
    let gh_path = fake_bin.join("gh");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$PWD\" >> {}\nsleep 0.25\ncat <<'JSON'\n{{\"number\":42,\"title\":\"Perf PR\",\"url\":\"https://example.com/pr/42\",\"state\":\"OPEN\",\"isDraft\":false,\"headRefName\":\"feat/perf\",\"baseRefName\":\"main\",\"author\":{{\"login\":\"alex\"}}}}\nJSON\n",
        shell_quote(&lookup_log.display().to_string())
    );
    fs::write(&gh_path, script).expect("fake gh should be written");
    let mut permissions = fs::metadata(&gh_path)
        .expect("fake gh should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&gh_path, permissions).expect("fake gh should be executable");
}

fn repo_shell_command(workdir: &Path, banner: &str) -> String {
    let script = format!(
        "cd {} && printf '%s\\n' {} && exec sleep 600",
        shell_quote(&workdir.display().to_string()),
        shell_quote(banner)
    );
    format!("sh -lc {}", shell_quote(&script))
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

fn wait_for_lookup_count(log_path: &Path, expected: usize, attempts: usize) {
    for _ in 0..attempts {
        if read_nonempty_lines(log_path).len() >= expected {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!(
        "expected at least {expected} fake gh lookups, got {:?}",
        read_nonempty_lines(log_path)
    );
}

fn read_nonempty_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
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
