use crate::support::tmux::TmuxFixture;
use foreman::adapters::tmux::{SystemTmuxBackend, TmuxAdapter};
use foreman::app::{AppState, SelectionTarget};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneHandle {
    pub session_name: String,
    pub pane_id: String,
    pub workdir: PathBuf,
}

#[derive(Debug)]
pub struct ReleaseHarness {
    fixture: TmuxFixture,
    config_root: PathBuf,
    log_dir: PathBuf,
    native_dir: PathBuf,
    bin_dir: PathBuf,
}

impl ReleaseHarness {
    pub fn new() -> Self {
        let fixture = TmuxFixture::new();
        let config_root = fixture.root_path().join("config");
        let log_dir = fixture.root_path().join("logs");
        let native_dir = fixture.root_path().join("native");
        let bin_dir = fixture.root_path().join("bin");

        fs::create_dir_all(config_root.join("foreman")).expect("config dir should exist");
        fs::create_dir_all(&log_dir).expect("log dir should exist");
        fs::create_dir_all(&native_dir).expect("native dir should exist");
        fs::create_dir_all(&bin_dir).expect("bin dir should exist");

        Self {
            fixture,
            config_root,
            log_dir,
            native_dir,
            bin_dir,
        }
    }

    pub fn fixture(&self) -> &TmuxFixture {
        &self.fixture
    }

    pub fn adapter(&self) -> TmuxAdapter<SystemTmuxBackend> {
        TmuxAdapter::new(SystemTmuxBackend::new(Some(
            self.fixture.socket_path().to_path_buf(),
        )))
    }

    pub fn latest_log_path(&self) -> PathBuf {
        self.log_dir.join("latest.log")
    }

    pub fn native_dir(&self) -> &Path {
        &self.native_dir
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_root.join("foreman/config.toml")
    }

    pub fn write_config(&self, contents: &str) {
        fs::write(self.config_file_path(), contents).expect("config should be written");
    }

    pub fn write_executable(&self, name: &str, contents: &str) {
        let path = self.bin_dir.join(name);
        fs::write(&path, contents).expect("script should be written");
        let mut permissions = fs::metadata(&path)
            .expect("script metadata should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("permissions should update");
    }

    pub fn write_atomic(&self, path: &Path, contents: &str) {
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, contents).expect("temp file should be written");
        fs::rename(&temp_path, path).expect("temp file should replace target");
    }

    pub fn write_native_signal(&self, pane_id: &str, contents: &str) {
        self.write_atomic(&self.native_dir.join(format!("{pane_id}.json")), contents);
    }

    pub fn wait_for_file_contains(&self, path: &Path, needle: &str) {
        for _ in 0..80 {
            if let Ok(contents) = fs::read_to_string(path) {
                if contents.contains(needle) {
                    return;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("file {} never contained {}", path.display(), needle);
    }

    pub fn wait_for_file_line_count(&self, path: &Path, expected: usize) {
        for _ in 0..80 {
            if self.nonempty_lines(path).len() >= expected {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!(
            "file {} never reached {} non-empty lines",
            path.display(),
            expected
        );
    }

    pub fn nonempty_lines(&self, path: &Path) -> Vec<String> {
        fs::read_to_string(path)
            .unwrap_or_default()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub fn wait_for_log_contains(&self, needle: &str) {
        let log_path = self.latest_log_path();
        for _ in 0..80 {
            let contents = fs::read_to_string(&log_path).unwrap_or_default();
            if contents.contains(needle) {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("log {} never contained {}", log_path.display(), needle);
    }

    pub fn log_occurrence_count(&self, needle: &str) -> usize {
        fs::read_to_string(self.latest_log_path())
            .unwrap_or_default()
            .matches(needle)
            .count()
    }

    pub fn wait_for_log_occurrence_count(&self, needle: &str, expected: usize) {
        let log_path = self.latest_log_path();
        for _ in 0..80 {
            let count = self.log_occurrence_count(needle);
            if count >= expected {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!(
            "log {} never reached {} occurrences of {}",
            log_path.display(),
            expected,
            needle
        );
    }

    pub fn create_workspace(&self, name: &str) -> PathBuf {
        let path = self.fixture.root_path().join(name);
        fs::create_dir_all(&path).expect("workspace dir should exist");
        path
    }

    pub fn create_git_repo(&self, name: &str) -> PathBuf {
        let path = self.create_workspace(name);
        std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&path)
            .status()
            .expect("git init should run");
        path
    }

    pub fn new_agent_session(
        &self,
        session_name: &str,
        workspace_name: &str,
        git_repo: bool,
        banner: &str,
        prefix: &str,
    ) -> PaneHandle {
        let workdir = if git_repo {
            self.create_git_repo(workspace_name)
        } else {
            self.create_workspace(workspace_name)
        };
        let pane_id = self
            .fixture
            .new_session(session_name, &agent_loop_command(&workdir, banner, prefix));
        self.fixture.wait_for_capture(&pane_id, banner);
        PaneHandle {
            session_name: session_name.to_string(),
            pane_id,
            workdir,
        }
    }

    pub fn split_agent_pane(
        &self,
        target: &str,
        workspace_name: &str,
        git_repo: bool,
        banner: &str,
        prefix: &str,
    ) -> PaneHandle {
        let workdir = if git_repo {
            self.create_git_repo(workspace_name)
        } else {
            self.create_workspace(workspace_name)
        };
        let pane_id = self
            .fixture
            .split_window(target, &agent_loop_command(&workdir, banner, prefix));
        self.fixture.wait_for_capture(&pane_id, banner);
        PaneHandle {
            session_name: target_session_name(target),
            pane_id,
            workdir,
        }
    }

    pub fn new_shell_session(
        &self,
        session_name: &str,
        workspace_name: &str,
        git_repo: bool,
        banner: &str,
    ) -> PaneHandle {
        let workdir = if git_repo {
            self.create_git_repo(workspace_name)
        } else {
            self.create_workspace(workspace_name)
        };
        let pane_id = self
            .fixture
            .new_session(session_name, &sleeping_shell_command(&workdir, banner));
        self.fixture.wait_for_capture(&pane_id, banner);
        PaneHandle {
            session_name: session_name.to_string(),
            pane_id,
            workdir,
        }
    }

    pub fn split_shell_pane(
        &self,
        target: &str,
        workspace_name: &str,
        git_repo: bool,
        banner: &str,
    ) -> PaneHandle {
        let workdir = if git_repo {
            self.create_git_repo(workspace_name)
        } else {
            self.create_workspace(workspace_name)
        };
        let pane_id = self
            .fixture
            .split_window(target, &sleeping_shell_command(&workdir, banner));
        self.fixture.wait_for_capture(&pane_id, banner);
        PaneHandle {
            session_name: target_session_name(target),
            pane_id,
            workdir,
        }
    }

    pub fn start_dashboard(&self, session_name: &str, extra_args: &[&str]) -> String {
        let mut args = vec![
            foreman_bin().to_string(),
            "--tmux-socket".to_string(),
            self.fixture.socket_path().display().to_string(),
        ];
        if !extra_args.contains(&"--poll-interval-ms") {
            args.push("--poll-interval-ms".to_string());
            args.push("150".to_string());
        }
        if !extra_args.contains(&"--capture-lines") {
            args.push("--capture-lines".to_string());
            args.push("20".to_string());
        }
        args.extend(extra_args.iter().map(|arg| (*arg).to_string()));

        let command = format!(
            "export FOREMAN_CONFIG_HOME={}; export FOREMAN_LOG_DIR={}; export PATH={}:$PATH; {}",
            shell_quote(&self.config_root.display().to_string()),
            shell_quote(&self.log_dir.display().to_string()),
            shell_quote(&self.bin_dir.display().to_string()),
            render_shell_command(&args)
        );
        self.fixture.new_session(
            session_name,
            &self.fixture.keep_alive_command(&command, "FOREMAN_EXITED"),
        )
    }

    pub fn wait_for_window_count(&self, session_name: &str, expected: usize) {
        for _ in 0..40 {
            let inventory = self
                .adapter()
                .load_inventory(20)
                .expect("inventory should load");
            let count = inventory
                .sessions
                .iter()
                .find(|session| session.name == session_name)
                .map(|session| session.windows.len())
                .unwrap_or_default();
            if count == expected {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("session {session_name} never reached {expected} windows");
    }

    pub fn wait_for_pane_removed(&self, pane_id: &str) {
        for _ in 0..40 {
            let inventory = self
                .adapter()
                .load_inventory(20)
                .expect("inventory should load");
            if inventory.pane(&pane_id.into()).is_none() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }

        panic!("pane {pane_id} never disappeared from inventory");
    }

    pub fn flash_label_for_target(&self, target: &SelectionTarget) -> String {
        let inventory = self
            .adapter()
            .load_inventory(20)
            .expect("inventory should load");
        let state = AppState::with_inventory(inventory);
        state
            .flash_label_for_target(target)
            .expect("flash label should exist")
    }
}

fn target_session_name(target: &str) -> String {
    target
        .split(':')
        .next()
        .expect("target should include session")
        .to_string()
}

fn shell_quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'\''"#))
}

fn render_shell_command(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn sleeping_shell_command(workdir: &Path, banner: &str) -> String {
    let script = format!(
        "cd {} && printf '%s\\n' {} && exec sleep 600",
        shell_quote(&workdir.display().to_string()),
        shell_quote(banner)
    );
    format!("sh -lc {}", shell_quote(&script))
}

fn agent_loop_command(workdir: &Path, banner: &str, prefix: &str) -> String {
    let script = format!(
        "cd {} && printf '%s\\n' {} && while IFS= read -r line; do printf '%s\\n' \"{}:$line\"; done",
        shell_quote(&workdir.display().to_string()),
        shell_quote(banner),
        prefix
    );
    format!("sh -lc {}", shell_quote(&script))
}

fn foreman_bin() -> &'static str {
    env!("CARGO_BIN_EXE_foreman")
}
