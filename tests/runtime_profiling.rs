mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};
use support::tmux::TmuxFixture;
use tempfile::tempdir;

const NAVIGATION_BURST_SESSION_COUNT: usize = 24;
const NAVIGATION_BURST_KEY_COUNT: usize = 48;
const MOVE_SELECTION_MAX_BUDGET_MS: u128 = 20;
const MOVE_SELECTION_P95_BUDGET_MS: u128 = 8;
const MOVE_SELECTION_BURST_BUDGET_MS: u128 = 1_500;
const OVERLAP_MOVE_SELECTION_MAX_BUDGET_MS: u128 = 25;
const OVERLAP_MOVE_SELECTION_P95_BUDGET_MS: u128 = 10;

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

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn navigation_burst_keeps_move_selection_latency_bounded_with_many_sessions() {
    let fixture = TmuxFixture::new();
    let root = tempdir().expect("temp dir should exist");
    let config_dir = root.path().join("config");
    let log_dir = root.path().join("logs");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");

    for index in 0..NAVIGATION_BURST_SESSION_COUNT {
        let name = format!("agent-{index:02}");
        let repo = root.path().join(&name);
        fs::create_dir_all(&repo).expect("repo dir should exist");
        let banner = match index % 3 {
            0 => "Claude Code ready",
            1 => "Codex CLI waiting for your input",
            _ => "Pi ready for the next task",
        };

        let pane_id = fixture.new_session(&name, &repo_shell_command(&repo, banner));
        fixture.wait_for_capture(&pane_id, banner);
    }

    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --debug --tmux-socket {} --poll-interval-ms 5000 --capture-lines 20 --no-notify",
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
    let latest_log_path = log_dir.join("latest.log");
    let burst_keys = vec!["j"; NAVIGATION_BURST_KEY_COUNT];
    let burst_started = Instant::now();
    fixture.send_keys(&dashboard_pane, &burst_keys);
    wait_for_action_timing_count(
        &latest_log_path,
        "move-selection",
        NAVIGATION_BURST_KEY_COUNT,
        120,
    );
    let burst_elapsed_ms = burst_started.elapsed().as_millis();

    let timings = read_action_timings(&latest_log_path, "move-selection");
    assert!(
        timings.len() >= NAVIGATION_BURST_KEY_COUNT,
        "expected at least {} move-selection timings, got {:?}",
        NAVIGATION_BURST_KEY_COUNT,
        timings
    );
    let recent_timings = &timings[timings.len() - NAVIGATION_BURST_KEY_COUNT..];
    let p95 = percentile(recent_timings, 95);
    let max = recent_timings.iter().copied().max().unwrap_or_default();
    let latest_log = fs::read_to_string(&latest_log_path).expect("latest log should be readable");
    let move_selection_line = latest_log
        .lines()
        .find(|line| line.contains("timing operation=action action=move-selection"))
        .expect("move-selection timing line should exist");
    let render_frame_line = latest_log
        .lines()
        .find(|line| line.contains("timing operation=render_frame"))
        .expect("render-frame timing line should exist");

    assert!(
        burst_elapsed_ms <= MOVE_SELECTION_BURST_BUDGET_MS,
        "expected navigation burst to finish within {MOVE_SELECTION_BURST_BUDGET_MS} ms, got {burst_elapsed_ms} ms with timings {recent_timings:?}"
    );
    assert!(
        p95 <= MOVE_SELECTION_P95_BUDGET_MS,
        "expected p95 move-selection timing <= {MOVE_SELECTION_P95_BUDGET_MS} ms, got {p95} ms from {recent_timings:?}"
    );
    assert!(
        max <= MOVE_SELECTION_MAX_BUDGET_MS,
        "expected max move-selection timing <= {MOVE_SELECTION_MAX_BUDGET_MS} ms, got {max} ms from {recent_timings:?}"
    );
    assert!(
        !latest_log
            .contains("slow_operation operation=action threshold_ms=40 action=move-selection"),
        "expected no slow move-selection warnings:\n{latest_log}"
    );
    assert!(
        latest_log.contains("timing operation=inventory_tmux"),
        "expected tmux inventory timing in latest log:\n{latest_log}"
    );
    assert!(
        latest_log.contains("timing operation=inventory_native"),
        "expected native overlay timing in latest log:\n{latest_log}"
    );
    assert!(
        move_selection_line.contains("visible_entries=")
            && move_selection_line.contains("selected_index=")
            && move_selection_line.contains("sidebar_scroll=")
            && move_selection_line.contains("viewport_rows="),
        "expected enriched move-selection timing fields, got:\n{move_selection_line}"
    );
    assert!(
        render_frame_line.contains("visible_entries=")
            && render_frame_line.contains("sidebar_scroll=")
            && render_frame_line.contains("viewport_rows="),
        "expected enriched render-frame timing fields, got:\n{render_frame_line}"
    );

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn inventory_refresh_reuses_offscreen_previews_under_crowded_load() {
    let fixture = TmuxFixture::new();
    let root = tempdir().expect("temp dir should exist");
    let config_dir = root.path().join("config");
    let log_dir = root.path().join("logs");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");

    for index in 0..NAVIGATION_BURST_SESSION_COUNT {
        let name = format!("agent-{index:02}");
        let repo = root.path().join(&name);
        fs::create_dir_all(&repo).expect("repo dir should exist");
        let banner = match index % 3 {
            0 => "Claude Code ready",
            1 => "Codex CLI waiting for your input",
            _ => "Pi ready for the next task",
        };

        let pane_id = fixture.new_session(&name, &repo_shell_command(&repo, banner));
        fixture.wait_for_capture(&pane_id, banner);
    }

    let dashboard_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --debug --tmux-socket {} --no-notify",
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
    let latest_log_path = log_dir.join("latest.log");
    wait_for_log_contains(&latest_log_path, "inventory_refresh_started", 120);
    let burst_keys = vec!["j"; NAVIGATION_BURST_KEY_COUNT];
    fixture.send_keys(&dashboard_pane, &burst_keys);
    wait_for_action_timing_count(
        &latest_log_path,
        "move-selection",
        NAVIGATION_BURST_KEY_COUNT,
        120,
    );
    wait_for_log_occurrence_count(&latest_log_path, "timing operation=inventory_tmux", 2, 120);
    let latest_log = fs::read_to_string(&latest_log_path).expect("latest log should be readable");
    let inventory_lines = inventory_tmux_lines(&latest_log);
    let refresh_line = inventory_lines
        .last()
        .expect("inventory tmux timing line should exist");
    let pane_records = timing_field(refresh_line, "pane_records");
    let captures = timing_field(refresh_line, "captures");
    let reused_previews = timing_field(refresh_line, "reused_previews");
    let timings = read_action_timings(&latest_log_path, "move-selection");
    let recent_timings = &timings[timings.len() - NAVIGATION_BURST_KEY_COUNT..];
    let p95 = percentile(recent_timings, 95);
    let max = recent_timings.iter().copied().max().unwrap_or_default();

    assert!(
        captures < pane_records,
        "expected staged refresh to capture fewer panes than discovered, got line:\n{refresh_line}"
    );
    assert!(
        reused_previews > 0,
        "expected staged refresh to reuse cached previews, got line:\n{refresh_line}"
    );
    assert!(
        p95 <= OVERLAP_MOVE_SELECTION_P95_BUDGET_MS,
        "expected overlapping refresh p95 <= {OVERLAP_MOVE_SELECTION_P95_BUDGET_MS}, got {p95} from timings {recent_timings:?}"
    );
    assert!(
        max <= OVERLAP_MOVE_SELECTION_MAX_BUDGET_MS,
        "expected overlapping refresh max <= {OVERLAP_MOVE_SELECTION_MAX_BUDGET_MS}, got {max} from timings {recent_timings:?}"
    );
    assert!(
        !latest_log
            .contains("slow_operation operation=action threshold_ms=40 action=move-selection"),
        "expected no slow move-selection warnings during staged refresh:\n{latest_log}"
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

fn wait_for_action_timing_count(log_path: &Path, action: &str, expected: usize, attempts: usize) {
    for _ in 0..attempts {
        if read_action_timings(log_path, action).len() >= expected {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!(
        "expected at least {expected} timing entries for action {action}, got {:?}",
        read_action_timings(log_path, action)
    );
}

fn wait_for_log_occurrence_count(log_path: &Path, needle: &str, expected: usize, attempts: usize) {
    for _ in 0..attempts {
        let count = read_nonempty_lines(log_path)
            .into_iter()
            .filter(|line| line.contains(needle))
            .count();
        if count >= expected {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!("expected at least {expected} log lines containing {needle}");
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

fn read_action_timings(path: &Path, action: &str) -> Vec<u128> {
    let prefix = format!("timing operation=action action={action} ");
    read_nonempty_lines(path)
        .into_iter()
        .filter(|line| line.contains(&prefix))
        .filter_map(|line| {
            line.split_whitespace()
                .find_map(|field| field.strip_prefix("total_ms="))
                .and_then(|value| value.parse::<u128>().ok())
        })
        .collect()
}

fn percentile(values: &[u128], percentile: usize) -> u128 {
    assert!(!values.is_empty(), "percentile requires at least one value");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).saturating_sub(1);
    sorted[rank.min(sorted.len().saturating_sub(1))]
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

fn inventory_tmux_lines(contents: &str) -> Vec<&str> {
    contents
        .lines()
        .filter(|line| line.contains("timing operation=inventory_tmux"))
        .collect()
}

fn timing_field(line: &str, field: &str) -> u128 {
    line.split_whitespace()
        .find_map(|segment| segment.strip_prefix(&format!("{field}=")))
        .and_then(|value| value.parse::<u128>().ok())
        .unwrap_or_else(|| panic!("missing {field} in timing line: {line}"))
}
