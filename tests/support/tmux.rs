use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

#[derive(Debug)]
pub struct TmuxFixture {
    _root: TempDir,
    socket_path: PathBuf,
}

impl TmuxFixture {
    pub fn new() -> Self {
        let root = tempfile::tempdir().expect("temp dir should exist");
        let socket_path = root.path().join("tmux.sock");

        let output = Command::new("tmux")
            .arg("-f")
            .arg("/dev/null")
            .arg("-S")
            .arg(&socket_path)
            .arg("start-server")
            .output()
            .expect("tmux should start");
        if !output.status.success() {
            panic!(
                "failed to start tmux server: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Self {
            _root: root,
            socket_path,
        }
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    pub fn new_session(&self, session_name: &str, command: &str) -> String {
        self.run_checked_retry(&[
            "new-session",
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            "-s",
            session_name,
            command,
        ])
    }

    #[allow(dead_code)]
    pub fn split_window(&self, target: &str, command: &str) -> String {
        self.run_checked_retry(&[
            "split-window",
            "-d",
            "-P",
            "-F",
            "#{pane_id}",
            "-t",
            target,
            command,
        ])
    }

    pub fn wait_for_capture(&self, target: &str, needle: &str) {
        self.wait_for_capture_attempts(target, needle, 40);
    }

    pub fn wait_for_capture_attempts(&self, target: &str, needle: &str, attempts: usize) {
        for _ in 0..attempts {
            let capture = self.capture(target);
            if capture.contains(needle) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("pane {target} never contained expected text: {needle}");
    }

    #[allow(dead_code)]
    pub fn wait_for_alt_capture(&self, target: &str, needle: &str) {
        self.wait_for_alt_capture_attempts(target, needle, 80);
    }

    pub fn wait_for_alt_capture_attempts(&self, target: &str, needle: &str, attempts: usize) {
        for _ in 0..attempts {
            let capture = self.capture_alt(target);
            if capture.contains(needle) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("pane {target} never contained expected alternate-screen text: {needle}");
    }

    pub fn shell_command(&self, line: &str) -> String {
        format!(
            "sh -lc \"printf '%s\\n' {}; exec sleep 60\"",
            shell_escape(line)
        )
    }

    #[allow(dead_code)]
    pub fn keep_alive_command(&self, command: &str, sentinel: &str) -> String {
        format!(
            "sh -lc '{}; printf \"%s\\n\" {}; exec sleep 60'",
            command.replace('\'', r#"'\''"#),
            sentinel
        )
    }

    #[allow(dead_code)]
    pub fn send_keys(&self, target: &str, keys: &[&str]) {
        let mut owned = vec![
            "send-keys".to_string(),
            "-t".to_string(),
            target.to_string(),
        ];
        owned.extend(keys.iter().map(|key| key.to_string()));
        let borrowed = owned.iter().map(String::as_str).collect::<Vec<_>>();
        self.run_checked(&borrowed);
    }

    #[allow(dead_code)]
    pub fn active_pane_in(&self, target: &str) -> String {
        self.run_checked(&[
            "list-panes",
            "-t",
            target,
            "-F",
            "#{pane_id}\t#{pane_active}",
        ])
        .lines()
        .find_map(|line| {
            let (pane_id, active) = line.split_once('\t')?;
            (active == "1").then(|| pane_id.to_string())
        })
        .expect("active pane should exist")
    }

    fn capture(&self, target: &str) -> String {
        self.run_checked(&["capture-pane", "-p", "-J", "-t", target, "-S", "-20"])
    }

    #[allow(dead_code)]
    pub fn capture_alt(&self, target: &str) -> String {
        let output = Command::new("tmux")
            .arg("-S")
            .arg(&self.socket_path)
            .args(["capture-pane", "-p", "-a", "-J", "-t", target, "-S", "-40"])
            .output()
            .expect("tmux command should run");

        if output.status.success() {
            let capture = String::from_utf8(output.stdout)
                .expect("tmux output should be utf-8")
                .trim()
                .to_string();
            if !capture.is_empty() {
                return capture;
            }

            return self.capture(target);
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no alternate screen") {
            return self.capture(target);
        }

        panic!(
            "tmux command failed: tmux -S {} capture-pane -p -a -J -t {} -S -40\nstderr: {}",
            self.socket_path.display(),
            target,
            stderr
        );
    }

    fn run_checked(&self, args: &[&str]) -> String {
        let output = Command::new("tmux")
            .arg("-S")
            .arg(&self.socket_path)
            .args(args)
            .output()
            .expect("tmux command should run");

        if !output.status.success() {
            panic!(
                "tmux command failed: tmux -S {} {}\nstderr: {}",
                self.socket_path.display(),
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        String::from_utf8(output.stdout)
            .expect("tmux output should be utf-8")
            .trim()
            .to_string()
    }

    fn run_checked_retry(&self, args: &[&str]) -> String {
        const RETRIES: usize = 5;

        let mut last_error = None;
        for attempt in 0..RETRIES {
            let output = Command::new("tmux")
                .arg("-S")
                .arg(&self.socket_path)
                .args(args)
                .output()
                .expect("tmux command should run");

            if output.status.success() {
                return String::from_utf8(output.stdout)
                    .expect("tmux output should be utf-8")
                    .trim()
                    .to_string();
            }

            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if attempt + 1 == RETRIES || !is_transient_server_error(&stderr) {
                last_error = Some(stderr);
                break;
            }

            thread::sleep(Duration::from_millis(50));
            last_error = Some(stderr);
        }

        panic!(
            "tmux command failed: tmux -S {} {}\nstderr: {}",
            self.socket_path.display(),
            args.join(" "),
            last_error.unwrap_or_else(|| "unknown tmux error".to_string())
        );
    }
}

fn shell_escape(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

fn is_transient_server_error(stderr: &str) -> bool {
    stderr.contains("server exited unexpectedly") || stderr.contains("no server running")
}

impl Drop for TmuxFixture {
    fn drop(&mut self) {
        let _ = Command::new("tmux")
            .arg("-S")
            .arg(&self.socket_path)
            .arg("kill-server")
            .output();
    }
}
