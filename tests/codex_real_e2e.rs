use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

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
        .args(["-a", "never", "exec"])
        .args(["-c", "features.codex_hooks=true"])
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
