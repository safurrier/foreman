use super::*;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn identity(uuid: &str) -> DisplayIdentity {
    DisplayIdentity {
        terminal_uuid: uuid.to_string(),
        tab_id: Some("tab-1".to_string()),
        window_id: Some("window-1".to_string()),
        diagnostic_title: Some("duplicate title".to_string()),
    }
}

#[test]
fn registry_path_is_owned_by_the_state_directory_above_logs() {
    let registry = SourceDisplayRegistry::for_log_dir(Path::new("/tmp/foreman/logs"));
    assert_eq!(
        registry.path(),
        Path::new("/tmp/foreman/source-displays.json")
    );
}

#[test]
fn registration_serializes_typed_provider_and_exact_identity() {
    let registration = SourceDisplayRegistration {
        source_id: "remote-dev".to_string(),
        provider: DisplayProviderKind::Ghostty,
        identity: identity("terminal-uuid"),
        captured_at_unix_ms: 100,
        updated_at_unix_ms: 101,
        ownership_handle: "opaque".to_string(),
    };
    let value = serde_json::to_value(&registration).unwrap();
    assert_eq!(value["provider"], "ghostty");
    assert_eq!(value["identity"]["terminalUuid"], "terminal-uuid");
    assert_eq!(value["identity"]["diagnosticTitle"], "duplicate title");
}

#[test]
fn replacement_invalidates_old_ownership_and_old_cleanup_cannot_delete_it() {
    let dir = tempfile::tempdir().unwrap();
    let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
    let source_id = SourceId::new("remote-dev");
    let first = registry
        .register(
            &source_id,
            DisplayProviderKind::Ghostty,
            identity("one"),
            10,
        )
        .unwrap();
    let second = registry
        .register(
            &source_id,
            DisplayProviderKind::Ghostty,
            identity("two"),
            20,
        )
        .unwrap();

    let error = registry
        .unregister(&source_id, &first.ownership_handle)
        .expect_err("old owner must not delete replacement");
    assert_eq!(error.code, "source.display.ownership-mismatch");
    assert_eq!(
        registry
            .get(&source_id)
            .unwrap()
            .unwrap()
            .identity
            .terminal_uuid,
        "two"
    );
    assert!(registry
        .unregister(&source_id, &second.ownership_handle)
        .unwrap());
    assert!(registry.get(&source_id).unwrap().is_none());
}

type AppleScriptCalls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

#[derive(Clone)]
struct RecordingAppleScript {
    output: Result<String, DisplayError>,
    calls: AppleScriptCalls,
}

impl AppleScriptExecutor for RecordingAppleScript {
    fn run(&self, script: &str, args: &[&str]) -> Result<String, DisplayError> {
        self.calls.lock().unwrap().push((
            script.to_string(),
            args.iter().map(|value| (*value).to_string()).collect(),
        ));
        self.output.clone()
    }
}

#[test]
fn ghostty_capture_parses_stable_ids_and_title_as_diagnostic_only() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let executor = RecordingAppleScript {
        output: Ok(
            "terminal-exact\u{001e}tab-exact\u{001e}window-exact\u{001e}same title\n".to_string(),
        ),
        calls: calls.clone(),
    };
    let provider = SystemDisplayProvider::with_executor(true, Box::new(executor));
    let captured = provider.capture(DisplayProviderKind::Ghostty).unwrap();
    assert_eq!(captured.terminal_uuid, "terminal-exact");
    assert_eq!(captured.tab_id.as_deref(), Some("tab-exact"));
    assert_eq!(captured.window_id.as_deref(), Some("window-exact"));
    assert_eq!(captured.diagnostic_title.as_deref(), Some("same title"));
    assert!(calls.lock().unwrap()[0].0.contains("focused terminal"));
}

#[test]
fn ghostty_focus_passes_only_exact_terminal_uuid_and_never_uses_title() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let executor = RecordingAppleScript {
        output: Ok("terminal-exact\n".to_string()),
        calls: calls.clone(),
    };
    let provider = SystemDisplayProvider::with_executor(true, Box::new(executor));
    let registration = SourceDisplayRegistration {
        source_id: "remote-dev".to_string(),
        provider: DisplayProviderKind::Ghostty,
        identity: identity("terminal-exact"),
        captured_at_unix_ms: 1,
        updated_at_unix_ms: 1,
        ownership_handle: "handle".to_string(),
    };
    provider.activate(&registration).unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(calls[0].1, vec!["terminal-exact"]);
    assert!(calls[0].0.contains("every terminal whose id is targetID"));
    assert!(calls[0].0.contains("focus targetTerminal"));
    assert!(!calls[0].0.contains("name of"));
}

#[test]
fn unsupported_platform_is_typed_and_does_not_run_applescript() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let executor = RecordingAppleScript {
        output: Ok(String::new()),
        calls: calls.clone(),
    };
    let provider = SystemDisplayProvider::with_executor(false, Box::new(executor));
    let error = provider
        .capture(DisplayProviderKind::Ghostty)
        .expect_err("non-macOS must fail clearly");
    assert_eq!(error.code, "source.display.platform-unsupported");
    assert!(calls.lock().unwrap().is_empty());
}

struct FakeProvider {
    activation: Result<(), DisplayError>,
    calls: Arc<Mutex<Vec<String>>>,
}

impl DisplayProviderAdapter for FakeProvider {
    fn capture(&self, _provider: DisplayProviderKind) -> Result<DisplayIdentity, DisplayError> {
        Ok(identity("captured"))
    }

    fn activate(&self, registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("provider:{}", registration.identity.terminal_uuid));
        self.activation.clone()
    }

    fn check(&self, _registration: &SourceDisplayRegistration) -> Result<(), DisplayError> {
        self.activation.clone()
    }
}

struct FakeCommandRunner {
    result: Result<(), DisplayError>,
    calls: Arc<Mutex<Vec<String>>>,
}

impl ActivationCommandRunner for FakeCommandRunner {
    fn run(
        &self,
        command: &str,
        _source_id: &str,
        _pane_id: &str,
        _timeout: Duration,
    ) -> Result<(), DisplayError> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("fallback:{command}"));
        self.result.clone()
    }
}

#[test]
fn resolver_attempts_registered_identity_before_command_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
    let source_id = SourceId::new("remote-dev");
    registry
        .register(
            &source_id,
            DisplayProviderKind::Ghostty,
            identity("terminal-exact"),
            1,
        )
        .unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let provider = FakeProvider {
        activation: Err(DisplayError::new(
            "source.display.unavailable",
            "closed terminal",
            true,
        )),
        calls: calls.clone(),
    };
    let runner = FakeCommandRunner {
        result: Ok(()),
        calls: calls.clone(),
    };

    let outcome = activate_registered_display_or_fallback(
        &registry,
        &provider,
        &runner,
        &source_id,
        "%42",
        Some("fallback-command"),
        Duration::from_secs(1),
    )
    .unwrap();

    assert!(outcome.ok);
    assert!(outcome.fallback_attempted);
    assert_eq!(
        outcome.code.as_deref(),
        Some("source.display.fallback-used")
    );
    assert_eq!(
        *calls.lock().unwrap(),
        vec!["provider:terminal-exact", "fallback:fallback-command"]
    );
}

#[test]
fn stale_or_closed_target_is_actionable_without_a_fallback() {
    let dir = tempfile::tempdir().unwrap();
    let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
    let source_id = SourceId::new("remote-dev");
    registry
        .register(
            &source_id,
            DisplayProviderKind::Ghostty,
            identity("closed"),
            1,
        )
        .unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let outcome = activate_registered_display_or_fallback(
        &registry,
        &FakeProvider {
            activation: Err(DisplayError::new(
                "source.display.unavailable",
                "registered Ghostty terminal is closed or missing",
                true,
            )),
            calls: calls.clone(),
        },
        &FakeCommandRunner {
            result: Ok(()),
            calls,
        },
        &source_id,
        "%42",
        None,
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(outcome.attempted);
    assert!(!outcome.ok);
    assert_eq!(outcome.code.as_deref(), Some("source.display.unavailable"));
    assert!(outcome.message.unwrap().contains("closed or missing"));
}

#[test]
fn activation_command_failure_preserves_stdout_diagnostic() {
    let error = SystemActivationCommandRunner::login_shell()
        .run(
            "printf 'activation target missing'; exit 7",
            "remote-dev",
            "%42",
            Duration::from_secs(1),
        )
        .expect_err("command should fail");

    assert_eq!(error.code, "source.display.activation-command-failed");
    assert!(error.message.contains("activation target missing"));
}

#[test]
fn activation_command_times_out_without_blocking_on_descendants() {
    let started = Instant::now();
    let error = SystemActivationCommandRunner::login_shell()
        .run("sleep 10", "remote-dev", "%42", Duration::from_millis(50))
        .expect_err("command should time out");

    assert_eq!(error.code, "source.display.activation-command-timeout");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn activation_command_collects_output_larger_than_a_pipe_buffer() {
    let error = SystemActivationCommandRunner::login_shell()
        .run(
            "yes diagnostic | head -c 200000; exit 7",
            "remote-dev",
            "%42",
            Duration::from_secs(2),
        )
        .expect_err("command should fail after producing output");

    assert_eq!(error.code, "source.display.activation-command-failed");
    assert!(error.message.contains("diagnostic"));
}

#[test]
fn activation_command_only_configuration_remains_supported() {
    let dir = tempfile::tempdir().unwrap();
    let registry = SourceDisplayRegistry::new(dir.path().join("displays.json"));
    let source_id = SourceId::new("legacy");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let outcome = activate_registered_display_or_fallback(
        &registry,
        &FakeProvider {
            activation: Ok(()),
            calls: calls.clone(),
        },
        &FakeCommandRunner {
            result: Ok(()),
            calls: calls.clone(),
        },
        &source_id,
        "%7",
        Some("legacy-command"),
        Duration::from_secs(1),
    )
    .unwrap();
    assert!(outcome.ok);
    assert_eq!(outcome.provider, None);
    assert_eq!(*calls.lock().unwrap(), vec!["fallback:legacy-command"]);
}
