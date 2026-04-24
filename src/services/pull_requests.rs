use serde::Deserialize;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const COMMAND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullRequestStatus {
    Open,
    Draft,
    Closed,
    Merged,
}

impl PullRequestStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Draft => "DRAFT",
            Self::Closed => "CLOSED",
            Self::Merged => "MERGED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestData {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub repository: String,
    pub branch: String,
    pub base_branch: String,
    pub author: String,
    pub status: PullRequestStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullRequestLookup {
    Unknown,
    Missing,
    Available(PullRequestData),
    Unavailable { message: String },
}

#[derive(Debug)]
pub enum PullRequestError {
    Io(std::io::Error),
    Unavailable(String),
    CommandFailed { command: String, stderr: String },
    CommandTimedOut { command: String, timeout_ms: u128 },
    Parse(String),
}

impl fmt::Display for PullRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Unavailable(message) => write!(f, "{message}"),
            Self::CommandFailed { command, stderr } => write!(f, "{command}: {stderr}"),
            Self::CommandTimedOut {
                command,
                timeout_ms,
            } => write!(f, "{command}: timed out after {timeout_ms} ms"),
            Self::Parse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for PullRequestError {}

impl From<std::io::Error> for PullRequestError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait PullRequestBackend {
    fn lookup(&self, workspace_path: &Path) -> Result<PullRequestLookup, PullRequestError>;
    fn open_in_browser(&self, url: &str) -> Result<(), PullRequestError>;
    fn copy_to_clipboard(&self, text: &str) -> Result<(), PullRequestError>;
}

#[derive(Debug, Clone, Default)]
pub struct PullRequestService<B> {
    backend: B,
}

impl<B> PullRequestService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: PullRequestBackend> PullRequestService<B> {
    pub fn lookup(&self, workspace_path: &Path) -> Result<PullRequestLookup, PullRequestError> {
        self.backend.lookup(workspace_path)
    }

    pub fn open_in_browser(&self, url: &str) -> Result<(), PullRequestError> {
        self.backend.open_in_browser(url)
    }

    pub fn copy_to_clipboard(&self, text: &str) -> Result<(), PullRequestError> {
        self.backend.copy_to_clipboard(text)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemPullRequestBackend;

impl SystemPullRequestBackend {
    pub fn new() -> Self {
        Self
    }
}

impl PullRequestBackend for SystemPullRequestBackend {
    fn lookup(&self, workspace_path: &Path) -> Result<PullRequestLookup, PullRequestError> {
        let repository_root = match resolve_repository_root(workspace_path) {
            Ok(repository_root) => repository_root,
            Err(PullRequestError::Unavailable(message))
                if is_missing_repository_message(&message) =>
            {
                return Ok(PullRequestLookup::Missing);
            }
            Err(PullRequestError::Unavailable(message)) => {
                return Ok(PullRequestLookup::Unavailable { message });
            }
            Err(error) => return Err(error),
        };

        let output = run_command(
            "gh",
            &[
                "pr",
                "view",
                "--json",
                "number,title,url,state,isDraft,headRefName,baseRefName,author",
            ],
            Some(&repository_root),
        );

        match output {
            Ok(stdout) => parse_pull_request_json(&stdout, &repository_root),
            Err(PullRequestError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(PullRequestLookup::Unavailable {
                    message: "GitHub CLI is not installed".to_string(),
                })
            }
            Err(PullRequestError::CommandFailed { stderr, .. })
                if is_missing_pull_request_message(&stderr) =>
            {
                Ok(PullRequestLookup::Missing)
            }
            Err(PullRequestError::CommandFailed { stderr, .. })
                if is_unavailable_pull_request_message(&stderr) =>
            {
                Ok(PullRequestLookup::Unavailable { message: stderr })
            }
            Err(error) => Err(error),
        }
    }

    fn open_in_browser(&self, url: &str) -> Result<(), PullRequestError> {
        run_fallback_commands(
            &[
                CommandSpec::new("open", &[url]),
                CommandSpec::new("xdg-open", &[url]),
                CommandSpec::new("cmd", &["/C", "start", "", url]),
            ],
            None,
        )
    }

    fn copy_to_clipboard(&self, text: &str) -> Result<(), PullRequestError> {
        run_fallback_commands_with_input(
            &[
                CommandSpec::new("pbcopy", &[]),
                CommandSpec::new("wl-copy", &[]),
                CommandSpec::new("xclip", &["-selection", "clipboard"]),
                CommandSpec::new("xsel", &["--clipboard", "--input"]),
                CommandSpec::new("cmd", &["/C", "clip"]),
            ],
            text,
            None,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct CommandSpec<'a> {
    program: &'a str,
    args: &'a [&'a str],
}

impl<'a> CommandSpec<'a> {
    const fn new(program: &'a str, args: &'a [&'a str]) -> Self {
        Self { program, args }
    }
}

#[derive(Debug, Deserialize)]
struct GhPullRequest {
    number: u64,
    title: String,
    url: String,
    state: String,
    #[serde(rename = "isDraft")]
    is_draft: bool,
    #[serde(rename = "headRefName")]
    head_ref_name: String,
    #[serde(rename = "baseRefName")]
    base_ref_name: String,
    author: GhAuthor,
}

#[derive(Debug, Deserialize)]
struct GhAuthor {
    login: String,
}

fn parse_pull_request_json(
    raw: &str,
    repository_root: &Path,
) -> Result<PullRequestLookup, PullRequestError> {
    let parsed: GhPullRequest = serde_json::from_str(raw)
        .map_err(|error| PullRequestError::Parse(format!("invalid gh pr view json: {error}")))?;
    let repository = repository_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(PullRequestLookup::Available(PullRequestData {
        number: parsed.number,
        title: parsed.title,
        url: parsed.url,
        repository,
        branch: parsed.head_ref_name,
        base_branch: parsed.base_ref_name,
        author: parsed.author.login,
        status: pull_request_status(&parsed.state, parsed.is_draft),
    }))
}

fn pull_request_status(state: &str, is_draft: bool) -> PullRequestStatus {
    if is_draft {
        return PullRequestStatus::Draft;
    }

    match state.to_ascii_uppercase().as_str() {
        "OPEN" => PullRequestStatus::Open,
        "MERGED" => PullRequestStatus::Merged,
        "CLOSED" => PullRequestStatus::Closed,
        _ => PullRequestStatus::Open,
    }
}

fn resolve_repository_root(workspace_path: &Path) -> Result<PathBuf, PullRequestError> {
    match run_command(
        "git",
        &["rev-parse", "--show-toplevel"],
        Some(workspace_path),
    ) {
        Ok(stdout) => Ok(PathBuf::from(stdout.trim())),
        Err(PullRequestError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Err(
            PullRequestError::Unavailable("git is not installed".to_string()),
        ),
        Err(PullRequestError::CommandFailed { stderr, .. })
            if is_missing_repository_message(&stderr) =>
        {
            Err(PullRequestError::Unavailable(stderr))
        }
        Err(error) => Err(error),
    }
}

fn run_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<String, PullRequestError> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    scrub_repo_context(&mut command);

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let rendered = render_command(program, args);
    let output = wait_with_timeout(command.spawn()?, rendered.clone(), COMMAND_TIMEOUT)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    Err(PullRequestError::CommandFailed {
        command: rendered,
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

fn scrub_repo_context(command: &mut Command) {
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_PREFIX",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GH_REPO",
    ] {
        command.env_remove(key);
    }
}

fn run_fallback_commands(
    candidates: &[CommandSpec<'_>],
    current_dir: Option<&Path>,
) -> Result<(), PullRequestError> {
    let mut last_error = None;

    for candidate in candidates {
        let mut command = Command::new(candidate.program);
        command.args(candidate.args);
        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }

        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let rendered = render_command(candidate.program, candidate.args);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                last_error = Some(PullRequestError::Io(error));
                continue;
            }
        };

        match wait_with_timeout(child, rendered.clone(), COMMAND_TIMEOUT) {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                last_error = Some(PullRequestError::CommandFailed {
                    command: rendered,
                    stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                });
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        PullRequestError::Unavailable("no browser opener is available".to_string())
    }))
}

fn run_fallback_commands_with_input(
    candidates: &[CommandSpec<'_>],
    input: &str,
    current_dir: Option<&Path>,
) -> Result<(), PullRequestError> {
    let mut last_error = None;

    for candidate in candidates {
        let mut command = Command::new(candidate.program);
        command.args(candidate.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        if let Some(current_dir) = current_dir {
            command.current_dir(current_dir);
        }

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                last_error = Some(PullRequestError::Io(error));
                continue;
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes())?;
        }

        let rendered = render_command(candidate.program, candidate.args);
        match wait_with_timeout(child, rendered.clone(), COMMAND_TIMEOUT) {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                last_error = Some(PullRequestError::CommandFailed {
                    command: rendered,
                    stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
                });
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        PullRequestError::Unavailable("no clipboard backend is available".to_string())
    }))
}

fn wait_with_timeout(
    mut child: Child,
    command: String,
    timeout: Duration,
) -> Result<Output, PullRequestError> {
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map_err(PullRequestError::Io);
        }

        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PullRequestError::CommandTimedOut {
                command,
                timeout_ms: timeout.as_millis(),
            });
        }

        thread::sleep(Duration::from_millis(10));
    }
}

fn render_command(program: &str, args: &[&str]) -> String {
    let mut rendered = String::from(program);
    for arg in args {
        rendered.push(' ');
        rendered.push_str(arg);
    }
    rendered
}

fn is_missing_repository_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("not a git repository")
}

fn is_missing_pull_request_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("no pull requests found")
        || lower.contains("could not find pull request")
        || lower.contains("no open pull requests")
}

fn is_unavailable_pull_request_message(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("not logged into any github hosts")
        || lower.contains("authentication")
        || lower.contains("http 401")
        || lower.contains("api rate limit exceeded")
}

#[cfg(test)]
mod tests {
    use super::{
        wait_with_timeout, PullRequestBackend, PullRequestData, PullRequestError,
        PullRequestLookup, PullRequestService, PullRequestStatus, SystemPullRequestBackend,
    };
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::rc::Rc;
    use std::time::Duration;
    use tempfile::Builder;

    #[derive(Debug, Clone)]
    struct FakePullRequestBackend {
        lookup_result: PullRequestLookup,
        looked_up_paths: Rc<RefCell<Vec<PathBuf>>>,
        opened_urls: Rc<RefCell<Vec<String>>>,
        copied_texts: Rc<RefCell<Vec<String>>>,
    }

    impl FakePullRequestBackend {
        fn available() -> Self {
            Self {
                lookup_result: PullRequestLookup::Available(PullRequestData {
                    number: 42,
                    title: "Tighten reducer invariants".to_string(),
                    url: "https://example.com/pr/42".to_string(),
                    repository: "foreman".to_string(),
                    branch: "feat/pr-state".to_string(),
                    base_branch: "main".to_string(),
                    author: "alex".to_string(),
                    status: PullRequestStatus::Open,
                }),
                looked_up_paths: Rc::new(RefCell::new(Vec::new())),
                opened_urls: Rc::new(RefCell::new(Vec::new())),
                copied_texts: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl PullRequestBackend for FakePullRequestBackend {
        fn lookup(
            &self,
            workspace_path: &Path,
        ) -> Result<PullRequestLookup, super::PullRequestError> {
            self.looked_up_paths
                .borrow_mut()
                .push(workspace_path.to_path_buf());
            Ok(self.lookup_result.clone())
        }

        fn open_in_browser(&self, url: &str) -> Result<(), super::PullRequestError> {
            self.opened_urls.borrow_mut().push(url.to_string());
            Ok(())
        }

        fn copy_to_clipboard(&self, text: &str) -> Result<(), super::PullRequestError> {
            self.copied_texts.borrow_mut().push(text.to_string());
            Ok(())
        }
    }

    #[test]
    fn service_delegates_lookup_and_auxiliary_actions() {
        let backend = FakePullRequestBackend::available();
        let service = PullRequestService::new(backend.clone());
        let workspace = Path::new("/tmp/worktree");

        let lookup = service.lookup(workspace).expect("lookup should succeed");
        service
            .open_in_browser("https://example.com/pr/42")
            .expect("open should succeed");
        service
            .copy_to_clipboard("https://example.com/pr/42")
            .expect("copy should succeed");

        assert!(matches!(lookup, PullRequestLookup::Available(_)));
        assert_eq!(
            backend.looked_up_paths.borrow().as_slice(),
            &[workspace.to_path_buf()]
        );
        assert_eq!(
            backend.opened_urls.borrow().as_slice(),
            &["https://example.com/pr/42".to_string()]
        );
        assert_eq!(
            backend.copied_texts.borrow().as_slice(),
            &["https://example.com/pr/42".to_string()]
        );
    }

    #[test]
    fn service_preserves_soft_unavailable_lookup_results() {
        let mut backend = FakePullRequestBackend::available();
        backend.lookup_result = PullRequestLookup::Unavailable {
            message: "GitHub CLI is not installed".to_string(),
        };
        let service = PullRequestService::new(backend);

        let lookup = service
            .lookup(Path::new("/tmp/worktree"))
            .expect("soft failures should still return lookup state");

        assert_eq!(
            lookup,
            PullRequestLookup::Unavailable {
                message: "GitHub CLI is not installed".to_string(),
            }
        );
    }

    #[test]
    fn system_backend_treats_non_repository_directory_as_missing() {
        let workspace = Builder::new()
            .prefix("foreman-pr-missing-")
            .tempdir_in("/tmp")
            .expect("temp dir should exist");
        let backend = SystemPullRequestBackend::new();

        let lookup = backend
            .lookup(workspace.path())
            .expect("non-repository paths should fail soft");

        assert_eq!(lookup, PullRequestLookup::Missing);
    }

    #[test]
    fn command_wait_times_out_slow_processes() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let child = command.spawn().expect("sleep command should spawn");

        let error = wait_with_timeout(child, "sh -c sleep 1".to_string(), Duration::from_millis(1))
            .expect_err("slow command should time out");

        assert!(matches!(error, PullRequestError::CommandTimedOut { .. }));
    }
}
