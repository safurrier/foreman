mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::thread;
use std::time::Duration;
use support::tmux::TmuxFixture;
use tempfile::tempdir;

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}

fn hook_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman-pi-hook")
}

fn write_executable(path: &std::path::Path, contents: &str) {
    fs::write(path, contents).expect("script should be written");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("permissions should update");
}

fn wait_for_file_contents(path: &std::path::Path, needle: &str) {
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

fn pi_is_available() -> bool {
    Command::new("pi")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore = "requires Pi auth, network, and explicit opt-in"]
fn real_pi_prompt_sent_through_dashboard_emits_completion_notification() {
    if std::env::var_os("FOREMAN_REAL_PI_E2E").is_none() {
        eprintln!("set FOREMAN_REAL_PI_E2E=1 to run this test");
        return;
    }

    if !pi_is_available() {
        eprintln!("pi CLI is not available in PATH");
        return;
    }

    let fixture = TmuxFixture::new();
    let temp_dir = tempdir().expect("temp dir should exist");
    let work_dir = temp_dir.path().join("workspace");
    let config_root = temp_dir.path().join("config");
    let log_dir = temp_dir.path().join("logs");
    let native_dir = temp_dir.path().join("native");
    let bin_dir = temp_dir.path().join("bin");
    let extension_dir = work_dir.join(".pi").join("extensions");
    let trace_file = temp_dir.path().join("pi-trace.log");
    let notification_file = temp_dir.path().join("notification.txt");

    fs::create_dir_all(&work_dir).expect("workspace dir should exist");
    fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
    fs::create_dir_all(&log_dir).expect("log dir should exist");
    fs::create_dir_all(&native_dir).expect("native dir should exist");
    fs::create_dir_all(&bin_dir).expect("bin dir should exist");
    fs::create_dir_all(&extension_dir).expect("extension dir should exist");

    Command::new("git")
        .args(["init", "-q"])
        .arg(&work_dir)
        .status()
        .expect("git should run");

    fs::write(
        config_root.join("foreman/config.toml"),
        r#"
[notifications]
enabled = true
cooldown_ticks = 1
backends = ["notify-send"]
active_profile = "completion-only"

[integrations.pi]
mode = "native"
"#,
    )
    .expect("config should be written");

    write_executable(
        &bin_dir.join("notify-send"),
        &format!(
            "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$FOREMAN_NOTIFY_KIND\" \"$FOREMAN_NOTIFY_TITLE\" \"$FOREMAN_NOTIFY_PANE_ID\" > \"{}\"\n",
            notification_file.display()
        ),
    );
    write_executable(
        &bin_dir.join("sh"),
        &format!(
            "#!/bin/sh\nexport PATH=\"{}:$PATH\"\nexec /bin/sh \"$@\"\n",
            bin_dir.display()
        ),
    );

    let extension_path = extension_dir.join("foreman.ts");
    fs::write(
        &extension_path,
        format!(
            r#"import type {{ ExtensionAPI }} from "@mariozechner/pi-coding-agent";
import {{ appendFileSync }} from "node:fs";
import {{ spawnSync }} from "node:child_process";

const hookBin = {hook_bin:?};
const nativeDir = {native_dir:?};
const traceFile = {trace_file:?};

function runHook(event: string) {{
  appendFileSync(traceFile, `${{event}}\n`);
  const args = ["--event", event, "--native-dir", nativeDir];
  const paneId = process.env.TMUX_PANE;
  if (paneId) {{
    args.push("--pane-id", paneId);
  }}
  const result = spawnSync(hookBin, args, {{ stdio: "inherit" }});
  if ((result.status ?? 1) !== 0) {{
    throw new Error(`foreman-pi-hook failed for ${{event}}`);
  }}
}}

export default function (pi: ExtensionAPI) {{
  pi.on("agent_start", async () => {{
    runHook("agent-start");
  }});
  pi.on("agent_end", async () => {{
    runHook("agent-end");
  }});
  pi.on("session_shutdown", async () => {{
    runHook("session-shutdown");
  }});
}}
"#,
            hook_bin = hook_bin(),
            native_dir = native_dir.display().to_string(),
            trace_file = trace_file.display().to_string(),
        ),
    )
    .expect("extension should be written");

    let pane_id = fixture.new_session(
        "alpha",
        &format!(
            "python3 -u -c {script}",
            script = shell_escape(&format!(
                "import os\nimport subprocess\nimport sys\nos.chdir({work_dir:?})\nos.environ['PATH'] = {bin_dir:?} + os.pathsep + os.environ.get('PATH', '')\nextension_path = {extension:?}\nprint('Pi hook loop ready', flush=True)\nfor line in sys.stdin:\n    prompt = line.rstrip('\\n')\n    print(f'PROMPT:{{prompt}}', flush=True)\n    subprocess.run(['pi', '--no-session', '-e', extension_path, '-p', prompt], check=True)\n    print('__PI_DONE__', flush=True)\n",
                work_dir = work_dir.display().to_string(),
                bin_dir = bin_dir.display().to_string(),
                extension = extension_path.display().to_string(),
            )),
        ),
    );
    fixture.wait_for_capture(&pane_id, "Pi hook loop ready");
    let beta_pane = fixture.new_session("beta", &fixture.shell_command("Pi ready"));
    fixture.wait_for_capture(&beta_pane, "Pi ready");

    let dashboard_command = format!(
        "PATH={}:$PATH FOREMAN_CONFIG_HOME={} FOREMAN_LOG_DIR={} {} --tmux-socket {} --poll-interval-ms 100 --capture-lines 20 --pi-native-dir {}",
        bin_dir.display(),
        config_root.display(),
        log_dir.display(),
        foreman_bin(),
        fixture.socket_path().display(),
        native_dir.display(),
    );
    let dashboard_pane = fixture.new_session(
        "dashboard",
        &fixture.keep_alive_command(&dashboard_command, "FOREMAN_EXITED"),
    );

    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman");
    fixture.wait_for_alt_capture(&dashboard_pane, "Foreman | NORMAL");

    fixture.send_keys(&dashboard_pane, &["j", "j"]);
    fixture.wait_for_alt_capture(&dashboard_pane, "Compose ->");

    for key in [
        "i", "R", "u", "n", "Space", "`", "s", "l", "e", "e", "p", "Space", "1", "`", "Space", "u",
        "s", "i", "n", "g", "Space", "b", "a", "s", "h", ",", "Space", "t", "h", "e", "n", "Space",
        "r", "e", "p", "l", "y", "Space", "w", "i", "t", "h", "Space", "e", "x", "a", "c", "t",
        "l", "y", "Space", "O", "K", ".", "C-s", "j",
    ] {
        fixture.send_keys(&dashboard_pane, &[key]);
    }
    fixture.wait_for_capture(
        &pane_id,
        "PROMPT:Run `sleep 1` using bash, then reply with exactly OK.",
    );

    wait_for_file_contents(&trace_file, "agent-start");
    wait_for_file_contents(&trace_file, "agent-end");
    wait_for_file_contents(&notification_file, "completion|Agent ready:");
    wait_for_file_contents(
        &native_dir.join(format!("{pane_id}.json")),
        r#""status":"idle""#,
    );

    fixture.send_keys(&dashboard_pane, &["q"]);
    fixture.wait_for_capture(&dashboard_pane, "FOREMAN_EXITED");
}

fn shell_escape(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}
