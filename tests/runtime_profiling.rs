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
const STARTUP_CACHE_WRITE_MAX_BUDGET_MS: u128 = 25;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn navigation_burst_limits_pull_request_lookup_churn() {
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
    fixture.wait_for_alt_capture(&dashboard_pane, "6 targets");
    fixture.send_keys(&dashboard_pane, &["j", "j", "j", "j", "j", "j"]);

    wait_for_lookup_count(&gh_lookup_log, 1, 60);
    let latest_log_path = log_dir.join("latest.log");
    wait_for_log_contains(&latest_log_path, "timing operation=pull_request_lookup", 60);
    let lookups = read_nonempty_lines(&gh_lookup_log);
    assert!(
        !lookups.is_empty(),
        "expected at least one fake gh lookup, got {lookups:?}"
    );
    assert!(
        lookups.len() <= 2,
        "expected at most bootstrap + one settled-selection lookup, got {lookups:?}"
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
    wait_for_log_contains(&latest_log_path, "timing operation=inventory_tmux", 120);
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
        "expected at least {NAVIGATION_BURST_KEY_COUNT} move-selection timings, got {timings:?}"
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

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn stable_inventory_writes_startup_cache_once_without_slowdowns() {
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
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --popup --debug --tmux-socket {} --poll-interval-ms 500 --capture-lines 20 --no-notify",
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
    wait_for_log_contains(
        &latest_log_path,
        "timing operation=startup_cache_write outcome=written",
        120,
    );
    wait_for_log_occurrence_count(&latest_log_path, "timing operation=inventory_tmux", 4, 160);
    let latest_log = fs::read_to_string(&latest_log_path).expect("latest log should be readable");
    let cache_write_lines = latest_log
        .lines()
        .filter(|line| line.contains("timing operation=startup_cache_write outcome=written"))
        .collect::<Vec<_>>();
    let max_elapsed_ms = cache_write_lines
        .iter()
        .map(|line| timing_field(line, "elapsed_ms"))
        .max()
        .unwrap_or_default();

    assert_eq!(
        cache_write_lines.len(),
        1,
        "expected one startup cache write for a stable inventory, got:\n{latest_log}"
    );
    assert!(
        max_elapsed_ms <= STARTUP_CACHE_WRITE_MAX_BUDGET_MS,
        "expected startup cache write <= {STARTUP_CACHE_WRITE_MAX_BUDGET_MS} ms, got {max_elapsed_ms} ms from {cache_write_lines:?}"
    );
    assert!(
        !latest_log.contains("slow_operation operation=startup_cache_write"),
        "expected no slow startup cache write warnings:\n{latest_log}"
    );

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn all_sources_popup_renders_local_rows_before_slow_ssh_source() {
    let fixture = TmuxFixture::new();
    let root = tempdir().expect("temp dir should exist");
    let config_dir = root.path().join("config");
    let log_dir = root.path().join("logs");
    let remote_done = root.path().join("remote-done");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");

    let repo = root.path().join("local-work");
    let fake_bin = root.path().join("fake-bin");
    fs::create_dir_all(&repo).expect("repo dir should exist");
    fs::create_dir_all(&fake_bin).expect("fake bin should exist");
    let fake_pi = fake_bin.join("pi");
    std::os::unix::fs::symlink("/bin/sleep", &fake_pi).expect("fake pi symlink should be created");
    let local_agent_command = format!(
        "sh -lc {}",
        shell_quote(&format!(
            "cd {} && PATH={}:$PATH exec pi 600",
            shell_quote(&repo.display().to_string()),
            shell_quote(&fake_bin.display().to_string())
        ))
    );
    let _pane_id = fixture.new_session("local-work", &local_agent_command);

    let local_log_dir = log_dir.join("local");
    fs::create_dir_all(&local_log_dir).expect("local log dir should exist");
    let local_command = format!(
        "FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --popup --debug --tmux-socket {} --poll-interval-ms 5000 --capture-lines 20 --no-notify",
        config_dir.join("local-config").display(),
        local_log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display()
    );
    let local_dashboard = fixture.new_session(
        "dashboard-local",
        &fixture.keep_alive_command(&local_command, "FOREMAN_LOCAL_EXITED"),
    );
    let local_started = Instant::now();
    fixture.wait_for_alt_capture(&local_dashboard, "2 targets");
    let local_first_target_ms = local_started.elapsed().as_millis();
    fixture.send_keys(&local_dashboard, &["q"]);
    fixture.wait_for_capture(&local_dashboard, "FOREMAN_LOCAL_EXITED");

    let fake_ssh = root.path().join("fake-ssh");
    write_slow_fake_ssh_source(&fake_ssh, &remote_done, 10_000);
    let source_config = root.path().join("source-config.toml");
    fs::write(
        &source_config,
        format!(
            r#"
[sources]
default_scope = "all"
query_timeout_ms = 12000

[sources.coder]
kind = "ssh"
label = "Coder"
host = "fake-coder"
foreman = "foreman"
ssh = "{}"

[sources.coder.display]
show_label = true
label = "Coder"
"#,
            fake_ssh.display()
        ),
    )
    .expect("source config should be written");

    let source_log_dir = log_dir.join("sources");
    fs::create_dir_all(&source_log_dir).expect("source log dir should exist");
    let source_command = format!(
        "FOREMAN_LOG_DIR={} {} --config-file {} --popup --debug --tmux-socket {} --sources all --poll-interval-ms 5000 --capture-lines 20 --no-notify",
        source_log_dir.display(),
        foreman_bin(),
        source_config.display(),
        fixture.socket_path().display()
    );
    let source_dashboard = fixture.new_session(
        "dashboard-sources",
        &fixture.keep_alive_command(&source_command, "FOREMAN_SOURCE_EXITED"),
    );
    let source_started = Instant::now();
    fixture.wait_for_alt_capture(&source_dashboard, "2 targets");
    let source_first_local_ms = source_started.elapsed().as_millis();
    assert!(
        !remote_done.exists(),
        "expected local rows to render before slow fake SSH source completed"
    );
    fixture.wait_for_alt_capture_attempts(&source_dashboard, "4 targets", 260);
    let source_remote_merge_ms = source_started.elapsed().as_millis();
    assert!(
        source_first_local_ms <= local_first_target_ms + 750,
        "expected all-source local first render to stay close to local-only baseline; local={local_first_target_ms}ms source={source_first_local_ms}ms"
    );
    assert!(
        source_remote_merge_ms >= 9_000,
        "remote merge should reflect fake SSH delay, got {source_remote_merge_ms}ms"
    );

    let latest_log = fs::read_to_string(source_log_dir.join("latest.log"))
        .expect("source latest log should be readable");
    assert!(
        latest_log.contains("timing operation=inventory_refresh outcome=loaded"),
        "expected source inventory timing logs:\n{latest_log}"
    );

    fixture.send_keys(&source_dashboard, &["q"]);
    fixture.wait_for_capture(&source_dashboard, "FOREMAN_SOURCE_EXITED");

    write_failing_fake_ssh_source(&fake_ssh);
    let failing_dashboard = fixture.new_session(
        "dashboard-sources-failing",
        &fixture.keep_alive_command(&source_command, "FOREMAN_SOURCE_FAILING_EXITED"),
    );
    fixture.wait_for_alt_capture(&failing_dashboard, "4 targets");
    wait_for_log_occurrence_count(
        &source_log_dir.join("latest.log"),
        "timing operation=inventory_refresh outcome=loaded",
        2,
        120,
    );
    let failing_capture = fixture.capture_alt(&failing_dashboard);
    assert!(
        failing_capture.contains("4 targets"),
        "expected cached remote rows to survive failed SSH refresh; capture:\n{failing_capture}"
    );
    fixture.send_keys(&failing_dashboard, &["q"]);
    fixture.wait_for_capture(&failing_dashboard, "FOREMAN_SOURCE_FAILING_EXITED");
}

#[test]
#[ignore = "heavy perf smoke for verify-ux and verify"]
fn popup_defers_all_source_merge_until_navigation_is_idle() {
    let fixture = TmuxFixture::new();
    let root = tempdir().expect("temp dir should exist");
    let config_dir = root.path().join("config");
    let log_dir = root.path().join("logs");
    let release_remote = root.path().join("release-remote");
    fs::create_dir_all(&config_dir).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");

    let repo = root.path().join("local-work");
    let fake_bin = root.path().join("fake-bin");
    fs::create_dir_all(&repo).expect("repo dir should exist");
    fs::create_dir_all(&fake_bin).expect("fake bin should exist");
    let fake_pi = fake_bin.join("pi");
    std::os::unix::fs::symlink("/bin/sleep", &fake_pi).expect("fake pi symlink should be created");
    let local_agent_command = format!(
        "sh -lc {}",
        shell_quote(&format!(
            "cd {} && PATH={}:$PATH exec pi 600",
            shell_quote(&repo.display().to_string()),
            shell_quote(&fake_bin.display().to_string())
        ))
    );
    let _pane_id = fixture.new_session("local-work", &local_agent_command);

    let fake_ssh = root.path().join("fake-ssh");
    write_released_fake_ssh_source(&fake_ssh, &release_remote);
    let source_config = root.path().join("source-config.toml");
    fs::write(
        &source_config,
        format!(
            r#"
[sources]
default_scope = "all"
query_timeout_ms = 12000

[sources.coder]
kind = "ssh"
label = "Coder"
host = "fake-coder"
foreman = "foreman"
ssh = "{}"

[sources.coder.display]
show_label = true
label = "Coder"
"#,
            fake_ssh.display()
        ),
    )
    .expect("source config should be written");

    let source_command = format!(
        "FOREMAN_LOG_DIR={} {} --config-file {} --popup --debug --tmux-socket {} --sources all --poll-interval-ms 5000 --capture-lines 20 --no-notify",
        log_dir.display(),
        foreman_bin(),
        source_config.display(),
        fixture.socket_path().display()
    );
    let dashboard = fixture.new_session(
        "dashboard-sources-deferred",
        &fixture.keep_alive_command(&source_command, "FOREMAN_DEFER_EXITED"),
    );
    fixture.wait_for_alt_capture(&dashboard, "2 targets");
    let latest_log_path = log_dir.join("latest.log");

    fs::write(&release_remote, "go").expect("release marker should be writable");
    let popup_key_count = 20;
    let burst_started = Instant::now();
    let socket_path = fixture.socket_path().to_path_buf();
    let dashboard_target = dashboard.clone();
    let sender = thread::spawn(move || {
        for _ in 0..popup_key_count {
            let status = std::process::Command::new("tmux")
                .arg("-S")
                .arg(&socket_path)
                .arg("send-keys")
                .arg("-t")
                .arg(&dashboard_target)
                .arg("j")
                .status()
                .expect("tmux send-keys should run");
            assert!(status.success(), "tmux send-keys should succeed");
            thread::sleep(Duration::from_millis(8));
        }
    });
    wait_for_action_timing_count(&latest_log_path, "move-selection", popup_key_count, 120);
    sender.join().expect("key sender should finish");
    let burst_elapsed_ms = burst_started.elapsed().as_millis();
    wait_for_log_contains(&latest_log_path, "inventory_refresh_deferred", 120);
    wait_for_log_contains(&latest_log_path, "inventory_refresh_deferred_apply", 120);
    fixture.wait_for_alt_capture(&dashboard, "4 targets");

    let latest_log = fs::read_to_string(&latest_log_path).expect("latest log should be readable");
    let timings = read_action_timings(&latest_log_path, "move-selection");
    let recent_timings = &timings[timings.len() - popup_key_count..];
    let p95 = percentile(recent_timings, 95);
    let max = recent_timings.iter().copied().max().unwrap_or_default();
    assert!(
        burst_elapsed_ms <= MOVE_SELECTION_BURST_BUDGET_MS,
        "expected key burst to finish within {MOVE_SELECTION_BURST_BUDGET_MS}ms while remote merge is deferred, got {burst_elapsed_ms}ms"
    );
    assert!(
        p95 <= MOVE_SELECTION_P95_BUDGET_MS,
        "expected p95 move-selection <= {MOVE_SELECTION_P95_BUDGET_MS}ms, got {p95} from {recent_timings:?}"
    );
    assert!(
        max <= MOVE_SELECTION_MAX_BUDGET_MS,
        "expected max move-selection <= {MOVE_SELECTION_MAX_BUDGET_MS}ms, got {max} from {recent_timings:?}"
    );
    assert!(
        !latest_log
            .contains("slow_operation operation=action threshold_ms=40 action=move-selection"),
        "expected no slow move-selection warnings while deferring source merge:\n{latest_log}"
    );

    fixture.send_keys(&dashboard, &["q"]);
    fixture.wait_for_capture(&dashboard, "FOREMAN_DEFER_EXITED");
}

fn write_released_fake_ssh_source(fake_ssh: &Path, release_marker: &Path) {
    let script = format!(
        "#!/bin/sh\nwhile [ ! -f {} ]; do sleep 0.01; done\nsleep 0.05\ncat <<'JSON'\n{{\"schemaVersion\":2,\"generatedAtUnixMs\":1,\"inventory\":{{\"totalSessions\":1,\"totalWindows\":1,\"totalPanes\":1,\"visibleSessions\":1,\"visibleWindows\":1,\"visiblePanes\":1}},\"entries\":[{{\"id\":\"source:local:pane:%42\",\"sourcePaneId\":\"source:local:pane:%42\",\"sourceId\":\"local\",\"sourceLabel\":\"Local\",\"sourceKind\":\"local\",\"paneId\":\"%42\",\"sessionId\":\"$remote\",\"sessionName\":\"0\",\"windowId\":\"@remote\",\"windowName\":\"zsh\",\"title\":\"remote-dots\",\"navigationTitle\":\"dots\",\"harness\":\"pi\",\"harnessLabel\":\"Pi\",\"status\":\"idle\",\"statusLabel\":\"IDLE\",\"statusSource\":\"compatibility\",\"integrationMode\":\"compatibility\",\"isAgent\":true,\"currentCommand\":\"pi\",\"runtimeCommand\":\"pi\",\"workingDir\":\"/home/discord/dots\",\"linkedRepository\":null,\"workspaceName\":\"dots\",\"preview\":\"remote ready\",\"previewProvenance\":\"captured\",\"activityScore\":1,\"statusRank\":3,\"lastActivityUnixMs\":1,\"lastStatusChangeUnixMs\":1,\"activeRunCount\":null,\"pullRequest\":null,\"extensionCards\":[]}}],\"diagnostics\":[]}}\nJSON\n",
        shell_quote(&release_marker.display().to_string())
    );
    fs::write(fake_ssh, script).expect("fake ssh should be written");
    let mut permissions = fs::metadata(fake_ssh)
        .expect("fake ssh should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(fake_ssh, permissions).expect("fake ssh should be executable");
}

fn write_slow_fake_ssh_source(fake_ssh: &Path, done_marker: &Path, sleep_ms: u64) {
    let sleep_seconds = format!("{}.{:03}", sleep_ms / 1000, sleep_ms % 1000);
    let script = format!(
        "#!/bin/sh\nsleep {sleep_seconds}\ntouch {}\ncat <<'JSON'\n{{\"schemaVersion\":2,\"generatedAtUnixMs\":1,\"inventory\":{{\"totalSessions\":1,\"totalWindows\":1,\"totalPanes\":1,\"visibleSessions\":1,\"visibleWindows\":1,\"visiblePanes\":1}},\"entries\":[{{\"id\":\"source:local:pane:%42\",\"sourcePaneId\":\"source:local:pane:%42\",\"sourceId\":\"local\",\"sourceLabel\":\"Local\",\"sourceKind\":\"local\",\"paneId\":\"%42\",\"sessionId\":\"$remote\",\"sessionName\":\"0\",\"windowId\":\"@remote\",\"windowName\":\"zsh\",\"title\":\"remote-dots\",\"navigationTitle\":\"dots\",\"harness\":\"pi\",\"harnessLabel\":\"Pi\",\"status\":\"idle\",\"statusLabel\":\"IDLE\",\"statusSource\":\"compatibility\",\"integrationMode\":\"compatibility\",\"isAgent\":true,\"currentCommand\":\"pi\",\"runtimeCommand\":\"pi\",\"workingDir\":\"/home/discord/dots\",\"linkedRepository\":null,\"workspaceName\":\"dots\",\"preview\":\"remote ready\",\"previewProvenance\":\"captured\",\"activityScore\":1,\"statusRank\":3,\"lastActivityUnixMs\":1,\"lastStatusChangeUnixMs\":1,\"activeRunCount\":null,\"pullRequest\":null,\"extensionCards\":[]}}],\"diagnostics\":[]}}\nJSON\n",
        shell_quote(&done_marker.display().to_string())
    );
    fs::write(fake_ssh, script).expect("fake ssh should be written");
    let mut permissions = fs::metadata(fake_ssh)
        .expect("fake ssh should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(fake_ssh, permissions).expect("fake ssh should be executable");
}

fn write_failing_fake_ssh_source(fake_ssh: &Path) {
    fs::write(
        fake_ssh,
        "#!/bin/sh\nprintf 'remote unavailable\\n' >&2\nexit 1\n",
    )
    .expect("failing fake ssh should be written");
    let mut permissions = fs::metadata(fake_ssh)
        .expect("fake ssh should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(fake_ssh, permissions).expect("fake ssh should be executable");
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
