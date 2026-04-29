use foreman::app::NotificationKind;
use foreman::config::LogVerbosity;
use foreman::services::logging::RunLogger;
use foreman::services::notifications::{
    CommandNotificationBackend, NotificationDecision, NotificationDecisionReason,
    NotificationDispatcher, NotificationRequest,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

fn write_executable_script(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).expect("script should be written");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("script permissions should update");
}

#[test]
fn shell_backed_notification_dispatcher_falls_back_and_logs_selection() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let fail_script = temp_dir.path().join("fail.sh");
    let success_script = temp_dir.path().join("success.sh");
    let capture_file = temp_dir.path().join("notification.txt");

    write_executable_script(&fail_script, "#!/bin/sh\nexit 1\n");
    write_executable_script(
        &success_script,
        "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"$1\"\n",
    );

    let mut dispatcher = NotificationDispatcher::new(vec![
        Box::new(CommandNotificationBackend::new(
            "primary",
            &fail_script,
            std::iter::empty::<String>(),
        )),
        Box::new(CommandNotificationBackend::new(
            "fallback",
            &success_script,
            [capture_file.display().to_string()],
        )),
    ]);
    let request = NotificationRequest {
        pane_id: "alpha:claude".into(),
        pane_title: "claude-main".to_string(),
        kind: NotificationKind::Completion,
        title: "Agent ready: claude-main".to_string(),
        subtitle: "claude-main".to_string(),
        body: "The agent returned to an idle state.".to_string(),
        audible: true,
        window_target: Some("alpha:0".to_string()),
        workspace_path: Some(temp_dir.path().to_path_buf()),
    };

    let receipt = dispatcher
        .dispatch(&request)
        .expect("fallback backend should succeed");

    let mut logger = RunLogger::start(&temp_dir.path().join("logs"), 2, LogVerbosity::Info)
        .expect("logger should start");
    logger
        .log_notification_decision(&NotificationDecision {
            pane_id: request.pane_id.clone(),
            kind: request.kind,
            reason: NotificationDecisionReason::WorkingBecameReady,
            request: Some(request.clone()),
        })
        .expect("decision log should succeed");
    logger
        .log_notification_backend_selected(&request, &receipt)
        .expect("backend selection log should succeed");

    let capture = fs::read_to_string(&capture_file).expect("capture file should exist");
    assert_eq!(
        capture.trim(),
        "Agent ready: claude-main|completion|alpha:claude"
    );

    let logs =
        fs::read_to_string(logger.summary().latest_path).expect("latest log should be readable");
    assert!(logs.contains("notification_decision"));
    assert!(logs.contains("notification_backend_selected"));
    assert!(logs.contains("backend=fallback"));
}

#[test]
fn inaudible_notification_dispatches_without_sound() {
    let temp_dir = tempdir().expect("temp dir should exist");
    let success_script = temp_dir.path().join("success.sh");
    let capture_file = temp_dir.path().join("notification.txt");

    write_executable_script(
        &success_script,
        "#!/bin/sh\nprintf 'sound=%s\\n' \"$FOREMAN_NOTIFY_SOUND\" > \"$1\"\n",
    );

    let mut dispatcher =
        NotificationDispatcher::new(vec![Box::new(CommandNotificationBackend::new(
            "capture",
            &success_script,
            [capture_file.display().to_string()],
        ))]);
    let request = NotificationRequest {
        pane_id: "alpha:claude".into(),
        pane_title: "claude-main".to_string(),
        kind: NotificationKind::Completion,
        title: "Agent ready: claude-main".to_string(),
        subtitle: "claude-main".to_string(),
        body: "The agent returned to an idle state.".to_string(),
        audible: false,
        window_target: Some("alpha:0".to_string()),
        workspace_path: Some(temp_dir.path().to_path_buf()),
    };

    dispatcher
        .dispatch(&request)
        .expect("notification should still dispatch");

    let capture = fs::read_to_string(&capture_file).expect("capture file should exist");
    assert_eq!(capture.trim(), "sound=");
}
