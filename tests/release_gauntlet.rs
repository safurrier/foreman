mod support;

use foreman::app::SelectionTarget;
use std::fs;
use std::thread;
use std::time::Duration;
use support::release::ReleaseHarness;
use support::tmux::TmuxFixture;

fn send_text(fixture: &TmuxFixture, target: &str, text: &str) {
    for ch in text.chars() {
        let key = match ch {
            ' ' => "Space".to_string(),
            _ => ch.to_string(),
        };
        fixture.send_keys(target, &[key.as_str()]);
    }
}

fn wait_for_window_named(harness: &ReleaseHarness, session_name: &str, window_name: &str) {
    let mut observed_names = Vec::new();
    for _ in 0..40 {
        let inventory = harness
            .adapter()
            .load_inventory(20)
            .expect("inventory should load");
        observed_names = inventory
            .sessions
            .iter()
            .find(|session| session.name == session_name)
            .map(|session| {
                session
                    .windows
                    .iter()
                    .map(|window| window.name.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let found = observed_names.iter().any(|name| name == window_name);
        if found {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!(
        "session {session_name} never exposed window {window_name}; observed windows: {observed_names:?}"
    );
}

#[test]
fn release_startup_navigation_gauntlet_proves_discovery_filters_and_help() {
    let harness = ReleaseHarness::new();
    let claude = harness.new_agent_session(
        "claudesess",
        "alphawork",
        true,
        "Claude Code ready",
        "CLAUDE",
    );
    let codex = harness.new_agent_session(
        "codexsess",
        "betawork",
        true,
        "Codex CLI waiting for your input",
        "CODEX",
    );
    let helper = harness.split_shell_pane(&codex.pane_id, "betashell", false, "beta helper");
    let _pi = harness.new_agent_session(
        "pisess",
        "gammawork",
        true,
        "Pi ready for the next task",
        "PI",
    );
    let _notes = harness.new_shell_session("notessess", "noteswork", false, "notes shell");

    let dashboard = harness.start_dashboard("dashboard", &["--no-notify"]);
    harness.fixture().resize_window("dashboard", 180, 48);

    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "theme=catppuccin");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "codexsess");
    harness.fixture().wait_for_alt_capture(&dashboard, "pisess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "notessess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "betashell");

    harness.fixture().send_keys(&dashboard, &["3"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "| INPUT |");
    harness.fixture().send_keys(&dashboard, &["2"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "| PREVIEW |");
    harness.fixture().send_keys(&dashboard, &["Tab"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "| INPUT |");
    harness.fixture().send_keys(&dashboard, &["Escape"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "| SIDEBAR |");

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "claudesess");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Foreman | NORMAL");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "› ▾ claudesess");

    harness.fixture().send_keys(&dashboard, &["f"]);
    harness
        .fixture()
        .wait_for_active_pane_in("claudesess", &claude.pane_id);

    harness.fixture().send_keys(&dashboard, &["j"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane:");
    harness.fixture().send_keys(&dashboard, &["f"]);
    harness
        .fixture()
        .wait_for_active_pane_in("claudesess", &claude.pane_id);

    harness.fixture().send_keys(&dashboard, &["j", "f"]);
    harness
        .fixture()
        .wait_for_active_pane_in("claudesess", &claude.pane_id);

    harness.fixture().send_keys(&dashboard, &["k", "k"]);
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "› ▸ claudesess");

    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "› ▾ claudesess");
    harness.fixture().send_keys(&dashboard, &["j", "i"]);
    send_text(harness.fixture(), &dashboard, "expanded");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&claude.pane_id, "CLAUDE:expanded");

    harness.fixture().send_keys(&dashboard, &["k", "f"]);
    harness
        .fixture()
        .wait_for_active_pane_in("claudesess", &claude.pane_id);

    harness.fixture().send_keys(&dashboard, &["?"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "Legend");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "f jumps tmux to the target pane");
    harness.fixture().send_keys(&dashboard, &["Escape"]);
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "Legend");

    harness.fixture().send_keys(&dashboard, &["t"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "theme=gruvbox");
    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "themekeep");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&claude.pane_id, "CLAUDE:themekeep");

    harness.fixture().send_keys(&dashboard, &["h"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "codexsess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "pisess");

    harness.fixture().send_keys(&dashboard, &["h"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "codexsess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "pisess");

    harness.fixture().send_keys(&dashboard, &["h"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "pisess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "codexsess");

    harness.fixture().send_keys(&dashboard, &["h"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "codexsess");
    harness.fixture().wait_for_alt_capture(&dashboard, "pisess");
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "Targets [gemini]");

    harness.fixture().send_keys(&dashboard, &["H"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "notessess");
    harness.fixture().send_keys(&dashboard, &["P"]);
    harness.fixture().wait_for_alt_capture(
        &dashboard,
        &helper.workdir.file_name().unwrap().to_string_lossy(),
    );

    harness.fixture().send_keys(&dashboard, &["q"]);
    harness
        .fixture()
        .wait_for_capture(&dashboard, "FOREMAN_EXITED");
}

#[test]
fn release_action_gauntlet_proves_search_flash_sort_and_pane_operations() {
    let harness = ReleaseHarness::new();
    let alpha =
        harness.new_agent_session("alphasess", "alphawork", true, "Claude Code ready", "ALPHA");
    let alpha_aux = harness.split_agent_pane(
        &alpha.pane_id,
        "alphaaux",
        true,
        "Codex CLI waiting for your input",
        "ALPHA2",
    );
    let beta = harness.new_agent_session(
        "betasess",
        "betawork",
        true,
        "Pi ready for the next task",
        "BETA",
    );
    let gamma =
        harness.new_agent_session("gammasess", "gammawork", true, "Claude Code ready", "GAMMA");

    let dashboard = harness.start_dashboard("dashboard", &["--no-notify"]);
    harness.fixture().resize_window("dashboard", 180, 48);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "theme=catppuccin");

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "betawork");
    harness.fixture().send_keys(&dashboard, &["Enter", "i"]);
    send_text(harness.fixture(), &dashboard, "search");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&beta.pane_id, "BETA:search");

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "alphawork");
    harness.fixture().send_keys(&dashboard, &["Escape"]);
    harness
        .fixture()
        .wait_for_alt_capture_not_contains(&dashboard, "Search");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Foreman | NORMAL");
    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "stillbeta");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&beta.pane_id, "BETA:stillbeta");

    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "one");
    harness.fixture().send_keys(&dashboard, &["C-j"]);
    send_text(harness.fixture(), &dashboard, "two");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&beta.pane_id, "BETA:one");
    harness
        .fixture()
        .wait_for_capture(&beta.pane_id, "BETA:two");

    let gamma_label =
        harness.flash_label_for_target(&SelectionTarget::Pane(gamma.pane_id.clone().into()));
    harness.fixture().send_keys(&dashboard, &["s"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "Flash");
    send_text(harness.fixture(), &dashboard, &gamma_label);
    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "flashjump");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&gamma.pane_id, "GAMMA:flashjump");

    let alpha_aux_label =
        harness.flash_label_for_target(&SelectionTarget::Pane(alpha_aux.pane_id.clone().into()));
    harness.fixture().send_keys(&dashboard, &["S"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "Flash");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Mode: jump+focus");
    send_text(harness.fixture(), &dashboard, &alpha_aux_label);
    harness
        .fixture()
        .wait_for_active_pane_in("alphasess", &alpha_aux.pane_id);

    harness.fixture().send_keys(&dashboard, &["o"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, " | attention | ");
    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "sortkeep");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&alpha_aux.pane_id, "ALPHA2:sortkeep");

    let current_window_name = harness
        .adapter()
        .load_inventory(20)
        .expect("inventory should load")
        .sessions
        .iter()
        .find(|session| session.name == "alphasess")
        .and_then(|session| session.windows.first())
        .map(|window| window.name.clone())
        .expect("alphasess window should exist");

    harness.fixture().send_keys(&dashboard, &["R"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "Rename");
    for _ in current_window_name.chars() {
        harness.fixture().send_keys(&dashboard, &["BSpace"]);
    }
    send_text(harness.fixture(), &dashboard, "alpharenamed");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    wait_for_window_named(&harness, "alphasess", "alpharenamed");

    harness.fixture().send_keys(&dashboard, &["N"]);
    harness.fixture().wait_for_alt_capture(&dashboard, "Spawn");
    send_text(harness.fixture(), &dashboard, "sleep 60");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness.wait_for_window_count("alphasess", 2);

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "betawork");
    harness.fixture().send_keys(&dashboard, &["Enter", "x"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Kill Pane");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness.wait_for_pane_removed(&beta.pane_id);

    harness.fixture().send_keys(&dashboard, &["q"]);
    harness
        .fixture()
        .wait_for_capture(&dashboard, "FOREMAN_EXITED");
}

#[test]
fn release_integration_gauntlet_proves_pr_notifications_and_graceful_degradation() {
    let harness = ReleaseHarness::new();
    let alpha = harness.new_agent_session(
        "claudesess",
        "alphawork",
        true,
        "Claude Code ready",
        "ALPHA",
    );
    let beta = harness.new_agent_session(
        "codexsess",
        "betawork",
        true,
        "Codex CLI waiting for your input",
        "BETA",
    );

    harness.write_config(
        r#"
[notifications]
enabled = true
cooldown_ticks = 0
backends = ["osascript", "notify-send"]
active_profile = "all"
"#,
    );

    let notification_file = harness.fixture().root_path().join("notifications.log");
    let browser_file = harness.fixture().root_path().join("browser.log");
    let clipboard_file = harness.fixture().root_path().join("clipboard.log");
    let gh_file = harness.fixture().root_path().join("gh-lookups.log");

    harness.write_executable("osascript", "#!/bin/sh\nexit 1\n");
    harness.write_executable(
        "notify-send",
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" >> \"{}\"\n",
            notification_file.display()
        ),
    );
    harness.write_executable(
        "open",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" > \"{}\"\n",
            browser_file.display()
        ),
    );
    harness.write_executable(
        "pbcopy",
        &format!("#!/bin/sh\ncat > \"{}\"\n", clipboard_file.display()),
    );
    harness.write_executable(
        "gh",
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$PWD\" >> \"{gh_log}\"\ncase \"$PWD\" in\n  *alphawork)\n    cat <<'JSON'\n{{\"number\":42,\"title\":\"Release gauntlet PR\",\"url\":\"https://example.com/pr/42\",\"state\":\"OPEN\",\"isDraft\":false,\"headRefName\":\"feat/gauntlet\",\"baseRefName\":\"main\",\"author\":{{\"login\":\"alex\"}}}}\nJSON\n    ;;\n  *betawork)\n    printf '%s\\n' 'authentication failed' >&2\n    exit 1\n    ;;\n  *)\n    printf '%s\\n' 'no pull requests found' >&2\n    exit 1\n    ;;\nesac\n",
            gh_log = gh_file.display()
        ),
    );

    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );

    let native_dir = harness.native_dir().display().to_string();
    let dashboard = harness.start_dashboard(
        "dashboard",
        &[
            "--debug",
            "--claude-native-dir",
            native_dir.as_str(),
            "--poll-interval-ms",
            "250",
        ],
    );
    harness.fixture().resize_window("dashboard", 180, 48);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Foreman | NORMAL");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "claudesess");

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "claudesess");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "claudesess");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane: ~ ✦ alphawork");
    harness.wait_for_log_contains("run_started");
    harness.wait_for_log_contains("pull_request_lookup workspace=");
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "pr=#42 OPEN");

    harness.fixture().send_keys(&dashboard, &["p"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Release gauntlet PR");
    harness.fixture().send_keys(&dashboard, &["O"]);
    harness.wait_for_file_contains(&browser_file, "https://example.com/pr/42");
    harness.fixture().send_keys(&dashboard, &["Y"]);
    harness.wait_for_file_contains(&clipboard_file, "https://example.com/pr/42");

    let refresh_marker = "inventory_refresh_started";
    let refresh_count = harness.log_occurrence_count(refresh_marker);
    harness.wait_for_log_occurrence_count(refresh_marker, refresh_count + 1);
    harness.write_native_signal(&alpha.pane_id, r#"{"status":"idle","activity_score":44}"#);
    thread::sleep(Duration::from_millis(75));
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );
    harness.wait_for_log_occurrence_count(refresh_marker, refresh_count + 2);
    assert!(
        harness.nonempty_lines(&notification_file).is_empty(),
        "brief idle loss should not notify immediately"
    );

    harness.write_native_signal(&alpha.pane_id, r#"{"status":"idle","activity_score":44}"#);
    harness.wait_for_file_line_count(&notification_file, 1);
    let notification_lines = harness.nonempty_lines(&notification_file);
    assert!(notification_lines[0].contains("completion|Agent ready:"));

    harness.fixture().send_keys(&dashboard, &["m"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "notify=MUTED");
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );
    thread::sleep(Duration::from_millis(300));
    harness.write_native_signal(&alpha.pane_id, r#"{"status":"idle","activity_score":44}"#);
    thread::sleep(Duration::from_millis(600));
    assert_eq!(harness.nonempty_lines(&notification_file).len(), 1);

    harness.fixture().send_keys(&dashboard, &["m", "n"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "notify=COMPLETE");
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane: ~ ✦ alphawork");
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"needs_attention","activity_score":90}"#,
    );
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane: ! ✦ alphawork");
    harness.wait_for_log_contains("kind=needs_attention action=suppress reason=profile_filtered");
    assert_eq!(harness.nonempty_lines(&notification_file).len(), 1);

    harness.fixture().send_keys(&dashboard, &["n"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "notify=ATTENTION");
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"working","activity_score":120}"#,
    );
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane: ~ ✦ alphawork");
    harness.write_native_signal(
        &alpha.pane_id,
        r#"{"status":"needs_attention","activity_score":90}"#,
    );
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "Target pane: ! ✦ alphawork");
    harness.wait_for_file_line_count(&notification_file, 2);
    let notification_lines = harness.nonempty_lines(&notification_file);
    assert!(notification_lines[1].contains("needs_attention|Needs attention:"));

    harness.fixture().send_keys(&dashboard, &["/"]);
    send_text(harness.fixture(), &dashboard, "betawork");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_alt_capture(&dashboard, "pr=UNAVAILABLE");
    harness.wait_for_log_contains("operator_alert source=pull_requests");
    harness.fixture().send_keys(&dashboard, &["i"]);
    send_text(harness.fixture(), &dashboard, "stillworks");
    harness.fixture().send_keys(&dashboard, &["Enter"]);
    harness
        .fixture()
        .wait_for_capture(&beta.pane_id, "BETA:stillworks");

    let log_contents =
        fs::read_to_string(harness.latest_log_path()).expect("latest log should be readable");
    assert!(log_contents.contains("notification_backend_selected"));
    assert!(log_contents.contains("backend=notify-send"));
    assert!(log_contents.contains("operator_alert source=pull_requests"));
    assert!(gh_file.exists());

    harness.fixture().send_keys(&dashboard, &["q"]);
    harness
        .fixture()
        .wait_for_capture(&dashboard, "FOREMAN_EXITED");
}
