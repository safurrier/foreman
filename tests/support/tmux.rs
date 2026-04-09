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
        self.run_checked(&[
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

    pub fn split_window(&self, target: &str, command: &str) -> String {
        self.run_checked(&[
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
        for _ in 0..40 {
            let capture = self.capture(target);
            if capture.contains(needle) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("pane {target} never contained expected text: {needle}");
    }

    pub fn shell_command(&self, line: &str) -> String {
        format!(
            "zsh -lc \"printf '%s\\n' '{}'; exec sleep 60\"",
            line.replace('\'', "")
        )
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
