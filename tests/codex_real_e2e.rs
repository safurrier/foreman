use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

const MIN_CODEX_HOOK_VERSION: (u64, u64, u64) = (0, 116, 0);

fn hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-codex-hook")
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).expect("script should be written");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .expect("script metadata should exist")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("script should be executable");
    }
}

fn wait_for_file_contents(path: &Path, needle: &str) {
    for _ in 0..400 {
        if let Ok(contents) = fs::read_to_string(path) {
            if contents.contains(needle) {
                return;
            }
        }
        thread::sleep(Duration::from_millis(50));
    }

    panic!(
        "file {} never contained expected text: {}",
        path.display(),
        needle
    );
}

fn codex_is_available() -> bool {
    Command::new("codex")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn parse_semver(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text
        .split(|ch: char| !(ch.is_ascii_digit() || ch == '.'))
        .find(|part| part.matches('.').count() >= 2)?
        .split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    Some((major, minor, patch))
}

fn codex_hook_support_error() -> Option<String> {
    let version_output = match Command::new("codex").arg("--version").output() {
        Ok(output) => output,
        Err(error) => return Some(format!("failed to execute codex --version: {error}")),
    };
    if !version_output.status.success() {
        return Some("codex --version failed".to_string());
    }

    let version_text = String::from_utf8_lossy(&version_output.stdout);
    let Some(parsed_version) = parse_semver(&version_text) else {
        return Some(format!(
            "could not parse codex version from: {}",
            version_text.trim()
        ));
    };

    if parsed_version < MIN_CODEX_HOOK_VERSION {
        return Some(format!(
            "codex on PATH is too old for UserPromptSubmit hooks: found {}.{}.{} but need >= {}.{}.{}",
            parsed_version.0,
            parsed_version.1,
            parsed_version.2,
            MIN_CODEX_HOOK_VERSION.0,
            MIN_CODEX_HOOK_VERSION.1,
            MIN_CODEX_HOOK_VERSION.2
        ));
    }

    let features_output = match Command::new("codex").args(["features", "list"]).output() {
        Ok(output) => output,
        Err(error) => return Some(format!("failed to execute codex features list: {error}")),
    };
    if !features_output.status.success() {
        return Some("codex features list failed".to_string());
    }

    let features = String::from_utf8_lossy(&features_output.stdout);
    if !features.contains("codex_hooks") {
        return Some(
            "codex on PATH does not expose the codex_hooks feature required by the real E2E"
                .to_string(),
        );
    }

    None
}

#[test]
#[ignore = "requires Codex auth, network, and explicit opt-in"]
fn real_codex_exec_emits_native_hook_signals() {
    if std::env::var_os("FOREMAN_REAL_CODEX_E2E").is_none() {
        eprintln!("set FOREMAN_REAL_CODEX_E2E=1 to run this test");
        return;
    }

    if !codex_is_available() {
        eprintln!("codex CLI is not available in PATH");
        return;
    }

    if let Some(message) = codex_hook_support_error() {
        panic!("{message}");
    }

    let temp_dir = tempdir().expect("temp dir should exist");
    let work_dir = temp_dir.path().join("workspace");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    let trace_file = temp_dir.path().join("hook-trace.log");
    fs::create_dir_all(work_dir.join(".codex")).expect("workspace dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");

    Command::new("git")
        .args(["init", "-q"])
        .arg(&work_dir)
        .status()
        .expect("git should run");

    let hook_wrapper = bin_dir.join("codex-hook-wrapper.sh");
    write_executable(
        &hook_wrapper,
        "#!/bin/sh\nset -eu\nEVENT=\"$1\"\nHOOK_BIN=\"$2\"\nNATIVE_DIR=\"$3\"\nTRACE_FILE=\"$4\"\nPANE_ID=\"$5\"\nTMP_INPUT=$(mktemp)\ncat > \"$TMP_INPUT\"\nprintf '%s\\n' \"$EVENT\" >> \"$TRACE_FILE\"\n\"$HOOK_BIN\" --native-dir \"$NATIVE_DIR\" --pane-id \"$PANE_ID\" < \"$TMP_INPUT\"\nSTATUS=$?\nrm -f \"$TMP_INPUT\"\nexit \"$STATUS\"\n",
    );

    fs::write(
        work_dir.join(".codex/hooks.json"),
        format!(
            r#"{{
  "hooks": {{
    "UserPromptSubmit": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} submit {hook_bin} {native_dir} {trace_file} %42"
          }}
        ]
      }}
    ],
    "Stop": [
      {{
        "hooks": [
          {{
            "type": "command",
            "command": "{hook_wrapper} stop {hook_bin} {native_dir} {trace_file} %42"
          }}
        ]
      }}
    ]
  }}
}}"#,
            hook_wrapper = hook_wrapper.display(),
            hook_bin = hook_bin(),
            native_dir = native_dir.display(),
            trace_file = trace_file.display(),
        ),
    )
    .expect("hooks config should be written");

    let output = Command::new("codex")
        .args(["--enable", "codex_hooks"])
        .args(["-a", "never", "exec"])
        .args(["-C"])
        .arg(&work_dir)
        .args(["--json", "--skip-git-repo-check"])
        .arg("Reply with exactly OK and nothing else.")
        .stdin(Stdio::null())
        .output()
        .expect("codex command should run");

    if !output.status.success() {
        panic!(
            "codex exec failed\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    wait_for_file_contents(&trace_file, "submit");
    wait_for_file_contents(&trace_file, "stop");

    let signal_path = native_dir.join("%42.json");
    wait_for_file_contents(&signal_path, r#""status":"idle""#);

    let jsonl = String::from_utf8(output.stdout).expect("stdout should be utf-8");
    assert!(jsonl.contains(r#""type":"thread.started""#));
    assert!(jsonl.contains(r#""type":"turn.completed""#));

    let signal = fs::read_to_string(&signal_path).expect("signal should exist");
    assert_eq!(signal, r#"{"status":"idle","activity_score":40}"#);
}
