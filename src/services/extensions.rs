use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(5);
const PROVIDER_ENV: &str = "FOREMAN_EXTENSION_PROVIDER_DIRS";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlExtensionCard {
    pub id: String,
    pub title: String,
    pub status: String,
    pub status_label: String,
    pub summary: String,
    #[serde(default)]
    pub rows: Vec<ControlExtensionRow>,
    #[serde(default)]
    pub actions: Vec<ControlExtensionAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlExtensionRow {
    pub label: String,
    pub value: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlExtensionAction {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionLookup {
    Missing,
    Available(Vec<ControlExtensionCard>),
    Unavailable { message: String },
}

#[derive(Debug)]
pub enum ExtensionError {
    Io(std::io::Error),
    CommandFailed { command: String, stderr: String },
    CommandTimedOut { command: String, timeout_ms: u128 },
    Parse(String),
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::CommandFailed { command, stderr } => write!(f, "{command}: {stderr}"),
            Self::CommandTimedOut {
                command,
                timeout_ms,
            } => write!(f, "{command}: timed out after {timeout_ms} ms"),
            Self::Parse(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ExtensionError {}

impl From<std::io::Error> for ExtensionError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

pub trait ControlExtensionProvider: Send + Sync {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn collect(&self, context: &ExtensionContext) -> Result<ExtensionLookup, ExtensionError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionContext {
    pub workspace_path: PathBuf,
    pub repository_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry<P> {
    providers: Vec<P>,
}

impl<P> ExtensionRegistry<P> {
    pub fn new(providers: Vec<P>) -> Self {
        Self { providers }
    }
}

impl<P: ControlExtensionProvider> ExtensionRegistry<P> {
    pub fn collect(&self, context: &ExtensionContext) -> Vec<ControlExtensionCard> {
        self.providers
            .iter()
            .filter_map(|provider| match provider.collect(context) {
                Ok(ExtensionLookup::Available(cards)) => Some(cards),
                Ok(ExtensionLookup::Missing) => None,
                Ok(ExtensionLookup::Unavailable { message }) => Some(vec![unavailable_card(
                    provider.id(),
                    provider.title(),
                    &message,
                )]),
                Err(error) => Some(vec![unavailable_card(
                    provider.id(),
                    provider.title(),
                    &error.to_string(),
                )]),
            })
            .flatten()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ExternalCommandProvider {
    manifest: ProviderManifest,
    manifest_dir: PathBuf,
}

impl ExternalCommandProvider {
    fn from_manifest_file(path: &Path) -> Result<Self, ExtensionError> {
        let contents = fs::read_to_string(path)?;
        let manifest = toml::from_str::<ProviderManifest>(&contents).map_err(|error| {
            ExtensionError::Parse(format!(
                "invalid provider manifest {}: {error}",
                path.display()
            ))
        })?;
        manifest.validate(path)?;
        let manifest_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        Ok(Self {
            manifest,
            manifest_dir,
        })
    }
}

impl ControlExtensionProvider for ExternalCommandProvider {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn title(&self) -> &str {
        &self.manifest.title
    }

    fn collect(&self, context: &ExtensionContext) -> Result<ExtensionLookup, ExtensionError> {
        if self.manifest.scope == ProviderScope::Repo && context.repository_root.is_none() {
            return Ok(ExtensionLookup::Missing);
        }

        let Some(command) = expand_token(&self.manifest.command, context) else {
            return Ok(ExtensionLookup::Missing);
        };
        let Some(args) = self
            .manifest
            .args
            .iter()
            .map(|arg| expand_token(arg, context))
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(ExtensionLookup::Missing);
        };
        let stdin = provider_stdin(context)?;
        let output = run_command(
            &command,
            &args,
            Some(&self.manifest_dir),
            Some(&stdin),
            Duration::from_millis(self.manifest.timeout_ms),
        )?;
        let response = serde_json::from_str::<ProviderResponse>(&output).map_err(|error| {
            ExtensionError::Parse(format!(
                "invalid provider {} json: {error}",
                self.manifest.id
            ))
        })?;
        if response.cards.is_empty() {
            Ok(ExtensionLookup::Missing)
        } else {
            Ok(ExtensionLookup::Available(response.cards))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct ProviderManifest {
    id: String,
    title: String,
    scope: ProviderScope,
    command: String,
    args: Vec<String>,
    timeout_ms: u64,
}

impl ProviderManifest {
    fn validate(&self, path: &Path) -> Result<(), ExtensionError> {
        if self.id.trim().is_empty() {
            return Err(ExtensionError::Parse(format!(
                "provider manifest {} is missing id",
                path.display()
            )));
        }
        if self.title.trim().is_empty() {
            return Err(ExtensionError::Parse(format!(
                "provider manifest {} is missing title",
                path.display()
            )));
        }
        if self.command.trim().is_empty() {
            return Err(ExtensionError::Parse(format!(
                "provider manifest {} is missing command",
                path.display()
            )));
        }
        Ok(())
    }
}

impl Default for ProviderManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            scope: ProviderScope::Repo,
            command: String::new(),
            args: Vec::new(),
            timeout_ms: DEFAULT_COMMAND_TIMEOUT.as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ProviderScope {
    Repo,
    Workspace,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProviderResponse {
    cards: Vec<ControlExtensionCard>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderContext<'a> {
    schema_version: u16,
    workspace_path: &'a str,
    repository_root: Option<&'a str>,
}

pub fn collect_workspace_extensions(
    workspaces: &[PathBuf],
) -> BTreeMap<PathBuf, Vec<ControlExtensionCard>> {
    workspaces
        .iter()
        .map(|workspace| {
            let context = ExtensionContext {
                workspace_path: workspace.clone(),
                repository_root: resolve_repository_root(workspace),
            };
            (workspace.clone(), context)
        })
        .map(|(workspace, context)| {
            thread::spawn(move || {
                let providers = discover_external_providers(&context);
                let registry = ExtensionRegistry::new(providers);
                let cards = registry.collect(&context);
                (workspace, cards)
            })
        })
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|handle| handle.join().ok())
        .collect()
}

fn discover_external_providers(context: &ExtensionContext) -> Vec<ExternalCommandProvider> {
    let mut providers_by_id = BTreeMap::<String, ExternalCommandProvider>::new();
    for manifest_path in provider_manifest_paths(context) {
        if let Ok(provider) = ExternalCommandProvider::from_manifest_file(&manifest_path) {
            providers_by_id.insert(provider.id().to_string(), provider);
        }
    }
    providers_by_id.into_values().collect()
}

fn provider_manifest_paths(context: &ExtensionContext) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = BTreeSet::new();
    for dir in provider_dirs(context) {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        let mut dir_paths = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        dir_paths.sort();
        for path in dir_paths {
            if seen.insert(path.clone()) {
                paths.push(path);
            }
        }
    }
    paths
}

fn provider_dirs(context: &ExtensionContext) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".config/foreman/providers"));
    }
    if let Some(root) = &context.repository_root {
        dirs.push(root.join(".foreman/providers"));
    }
    if let Some(value) = std::env::var_os(PROVIDER_ENV) {
        dirs.extend(std::env::split_paths(&value));
    }
    dirs
}

fn unavailable_card(id: &str, title: &str, message: &str) -> ControlExtensionCard {
    ControlExtensionCard {
        id: id.to_string(),
        title: title.to_string(),
        status: "unavailable".to_string(),
        status_label: "UNAVAILABLE".to_string(),
        summary: message.to_string(),
        rows: vec![ControlExtensionRow {
            label: "Provider".to_string(),
            value: message.to_string(),
            status: Some("fail".to_string()),
        }],
        actions: Vec::new(),
    }
}

fn resolve_repository_root(path: &Path) -> Option<PathBuf> {
    let mut cursor = if path.is_file() { path.parent()? } else { path };
    loop {
        if cursor.join(".git").exists() {
            return Some(cursor.to_path_buf());
        }
        cursor = cursor.parent()?;
    }
}

fn expand_token(template: &str, context: &ExtensionContext) -> Option<String> {
    let repository_root = context
        .repository_root
        .as_ref()
        .map(|path| path_string(path));
    if template.contains("{repo}") && repository_root.is_none() {
        return None;
    }
    Some(
        template
            .replace("{workspace}", &path_string(&context.workspace_path))
            .replace("{repo}", repository_root.as_deref().unwrap_or("")),
    )
}

fn provider_stdin(context: &ExtensionContext) -> Result<String, ExtensionError> {
    let workspace = path_string(&context.workspace_path);
    let repository_root = context
        .repository_root
        .as_ref()
        .map(|path| path_string(path));
    serde_json::to_string(&ProviderContext {
        schema_version: 1,
        workspace_path: &workspace,
        repository_root: repository_root.as_deref(),
    })
    .map_err(|error| ExtensionError::Parse(format!("failed to encode provider context: {error}")))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn run_command(
    program: &str,
    args: &[String],
    current_dir: Option<&Path>,
    stdin: Option<&str>,
    timeout: Duration,
) -> Result<String, ExtensionError> {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }
    let mut stdout = create_output_file("stdout")?;
    let mut stderr = create_output_file("stderr")?;
    command
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::from(stdout.file.try_clone()?))
        .stderr(Stdio::from(stderr.file.try_clone()?));
    let mut child = command.spawn()?;
    if let Some(stdin) = stdin {
        if let Some(mut child_stdin) = child.stdin.take() {
            match child_stdin.write_all(stdin.as_bytes()) {
                Ok(()) => {}
                Err(error) if error.kind() == ErrorKind::BrokenPipe => {}
                Err(error) => return Err(ExtensionError::Io(error)),
            }
        }
    }
    let status = wait_with_timeout(child, timeout, command_line(program, args))?;
    let output = stdout.read_contents()?;
    let error_output = stderr.read_contents()?;
    if status.success() {
        Ok(output)
    } else {
        Err(ExtensionError::CommandFailed {
            command: command_line(program, args),
            stderr: error_output.trim().to_string(),
        })
    }
}

struct TempOutputFile {
    path: PathBuf,
    file: fs::File,
}

impl TempOutputFile {
    fn read_contents(&mut self) -> Result<String, ExtensionError> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut contents = String::new();
        self.file.read_to_string(&mut contents)?;
        Ok(contents)
    }
}

impl Drop for TempOutputFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn create_output_file(label: &str) -> Result<TempOutputFile, ExtensionError> {
    let mut attempts = 0u32;
    loop {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "foreman-extension-{label}-{}-{nonce}-{attempts}.log",
            std::process::id()
        ));
        match fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create_new(true)
            .open(&path)
        {
            Ok(file) => return Ok(TempOutputFile { path, file }),
            Err(error) if error.kind() == ErrorKind::AlreadyExists && attempts < 10 => {
                attempts += 1;
            }
            Err(error) => return Err(ExtensionError::Io(error)),
        }
    }
}

fn wait_with_timeout(
    mut child: Child,
    timeout: Duration,
    command: String,
) -> Result<ExitStatus, ExtensionError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ExtensionError::CommandTimedOut {
                command,
                timeout_ms: timeout.as_millis(),
            });
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn command_line(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &std::ffi::OsStr) -> Self {
            let original = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, original }
        }

        fn remove(key: &'static str) -> Self {
            let original = std::env::var_os(key);
            std::env::remove_var(key);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(original) = &self.original {
                std::env::set_var(self.key, original);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn repository_root_walks_up_to_git_directory() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let nested = temp.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();

        assert_eq!(
            resolve_repository_root(&nested),
            Some(temp.path().to_path_buf())
        );
    }

    #[test]
    fn manifest_provider_expands_repo_context_and_parses_cards() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _provider_guard = EnvVarGuard::remove(PROVIDER_ENV);
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let _home_guard = EnvVarGuard::set("HOME", home.as_os_str());
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let providers_dir = temp.path().join(".foreman/providers");
        std::fs::create_dir_all(&providers_dir).unwrap();
        let script = temp.path().join("provider.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\ncat <<'JSON'\n{\"cards\":[{\"id\":\"demo\",\"title\":\"Demo\",\"status\":\"ready\",\"statusLabel\":\"READY\",\"summary\":\"ok\"}]}\nJSON\n",
        )
        .unwrap();
        make_executable(&script);
        std::fs::write(
            providers_dir.join("demo.toml"),
            format!(
                "id = \"demo\"\ntitle = \"Demo\"\ncommand = \"{}\"\nargs = [\"--repo\", \"{{repo}}\"]\n",
                script.display()
            ),
        )
        .unwrap();

        let cards = collect_workspace_extensions(&[temp.path().to_path_buf()]);
        let cards = cards.get(temp.path()).unwrap();

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "demo");
        assert_eq!(cards[0].summary, "ok");
    }

    #[test]
    fn harness_kit_example_provider_finds_hk_in_local_bin_with_sparse_app_environment() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        std::fs::create_dir(repo.join(".git")).unwrap();
        let brief = serde_json::json!({
            "active_work": "",
            "sync_status": "no-active-work",
            "handoff_export": {
                "state": "no-active-work",
                "commands": {
                    "start": format!("hk start demo-work --plan 'Describe the intended change' --target {}", path_string(&repo))
                }
            }
        });
        let status = serde_json::json!({
            "active_work": "none",
            "ready_status": "not-started",
            "sync_status": "no-active-work",
            "checks": [],
            "next_actions": []
        });

        let (response, calls) =
            run_hk_example_provider_with_fake_hk(temp.path(), &repo, &brief, &status);

        assert_eq!(response.cards.len(), 1);
        assert_eq!(response.cards[0].status_label, "NO WORK");
        assert!(response.cards[0].rows.is_empty());
        assert_eq!(action_labels(&response.cards[0]), vec!["Copy start"]);
        assert_eq!(calls, vec!["brief", "status"]);
    }

    #[test]
    fn harness_kit_example_provider_surfaces_missing_handoff_export_copy_actions() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        std::fs::create_dir(repo.join(".git")).unwrap();
        let readme = repo.join(".ai/hk/2026-05-12-120000-demo/README.md");
        let brief = serde_json::json!({
            "active_work": "2026-05-12-120000-demo",
            "sync_status": "needs-sync",
            "handoff_export": {
                "state": "missing",
                "exists": false,
                "fresh": false,
                "readme_exists": false,
                "readme_path": path_string(&readme),
                "commands": {
                    "generate": format!("hk export --format handoff-dir --target {}", path_string(&repo)),
                    "preview": format!("hk handoff --target {} --json", path_string(&repo)),
                    "check": format!("hk export --format handoff-dir --check --target {} --json", path_string(&repo))
                }
            }
        });
        let status = active_status();

        let (response, calls) =
            run_hk_example_provider_with_fake_hk(temp.path(), &repo, &brief, &status);
        let card = &response.cards[0];

        assert_eq!(calls, vec!["brief", "status"]);
        assert_eq!(row_value(card, "Export"), Some("missing"));
        let labels = action_labels(card);
        assert!(labels.contains(&"Copy export"));
        assert!(labels.contains(&"Copy handoff preview"));
        assert!(!labels.contains(&"Open handoff"));
        assert!(!labels.contains(&"Open stale handoff"));
    }

    #[test]
    fn harness_kit_example_provider_surfaces_stale_and_invalid_export_actions() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        std::fs::create_dir(repo.join(".git")).unwrap();
        let readme = repo.join(".ai/hk/2026-05-12-120000-demo/README.md");
        std::fs::create_dir_all(readme.parent().unwrap()).unwrap();
        std::fs::write(&readme, "# Stale Handoff\n").unwrap();
        let mut brief = serde_json::json!({
            "active_work": "2026-05-12-120000-demo",
            "sync_status": "needs-sync",
            "handoff_export": {
                "state": "stale",
                "exists": true,
                "fresh": false,
                "readme_exists": true,
                "readme_path": path_string(&readme),
                "commands": {
                    "generate": format!("hk export --format handoff-dir --target {}", path_string(&repo)),
                    "preview": format!("hk handoff --target {} --json", path_string(&repo)),
                    "check": format!("hk export --format handoff-dir --check --target {} --json", path_string(&repo))
                }
            }
        });
        let status = active_status();

        let (response, _calls) =
            run_hk_example_provider_with_fake_hk(temp.path(), &repo, &brief, &status);
        let card = &response.cards[0];
        assert_eq!(row_value(card, "Export"), Some("stale"));
        assert_eq!(
            card.actions
                .iter()
                .find(|action| action.label == "Open stale handoff")
                .map(|action| action.value.as_str()),
            Some(path_string(&readme).as_str())
        );
        assert!(action_labels(card).contains(&"Copy export"));

        brief["handoff_export"]["state"] = serde_json::Value::String("invalid".to_owned());
        brief["handoff_export"]["readme_exists"] = serde_json::Value::Bool(false);
        let (response, _calls) =
            run_hk_example_provider_with_fake_hk(temp.path(), &repo, &brief, &status);
        let card = &response.cards[0];
        assert_eq!(row_value(card, "Export"), Some("invalid"));
        assert!(action_labels(card).contains(&"Copy export"));
        assert!(action_labels(card).contains(&"Copy export check"));
    }

    #[test]
    fn harness_kit_example_provider_opens_fresh_handoff_export_readme() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        std::fs::create_dir(repo.join(".git")).unwrap();
        let readme = repo.join(".ai/hk/2026-05-12-120000-demo/README.md");
        std::fs::create_dir_all(readme.parent().unwrap()).unwrap();
        std::fs::write(&readme, "# Handoff\n").unwrap();
        let brief = serde_json::json!({
            "active_work": "2026-05-12-120000-demo",
            "sync_status": "synced",
            "handoff_export": {
                "state": "fresh",
                "exists": true,
                "fresh": true,
                "readme_exists": true,
                "readme_path": path_string(&readme),
                "commands": {
                    "generate": format!("hk export --format handoff-dir --target {}", path_string(&repo)),
                    "preview": format!("hk handoff --target {} --json", path_string(&repo)),
                    "check": format!("hk export --format handoff-dir --check --target {} --json", path_string(&repo))
                }
            }
        });
        let status = active_status();

        let (response, _calls) =
            run_hk_example_provider_with_fake_hk(temp.path(), &repo, &brief, &status);
        let card = &response.cards[0];
        let action = card
            .actions
            .iter()
            .find(|action| action.label == "Open handoff")
            .expect("Open handoff action");

        assert_eq!(action.kind, "open-file");
        assert_eq!(action.value, path_string(&readme));
        assert_eq!(row_value(card, "Export"), Some("fresh"));
    }

    fn active_status() -> serde_json::Value {
        serde_json::json!({
            "active_work": "2026-05-12-120000-demo",
            "ready_status": "not-ready",
            "sync_status": "needs-sync",
            "phase": "finalizing",
            "state_dir": "/tmp/hk-state",
            "checks": [
                {"id": "validation", "status": "fail", "message": "validation evidence is stale for current diff"},
                {"id": "review", "status": "pass", "message": "external-enough review recorded"}
            ],
            "next_actions": ["validation: run `hk validate --check fast-gate --why 'Fast gate passes' -- mise run check` using the matching native command"]
        })
    }

    fn run_hk_example_provider_with_fake_hk(
        temp: &Path,
        repo: &Path,
        brief: &serde_json::Value,
        status: &serde_json::Value,
    ) -> (ProviderResponse, Vec<String>) {
        let home = temp.join("home");
        let local_bin = home.join(".local/bin");
        std::fs::create_dir_all(&local_bin).unwrap();
        let calls = temp.join("hk-calls.log");
        let brief_path = temp.join("brief.json");
        let status_path = temp.join("status.json");
        std::fs::write(&brief_path, serde_json::to_string(brief).unwrap()).unwrap();
        std::fs::write(&status_path, serde_json::to_string(status).unwrap()).unwrap();
        let fake_hk = local_bin.join("hk");
        std::fs::write(
            &fake_hk,
            format!(
                r#"#!/bin/sh
echo "$1" >> {}
case "$1" in
  brief) cat {} ;;
  status) cat {} ;;
  ready|export|sync) echo "unexpected mutating/extra command: $1" >&2; exit 64 ;;
  *) echo "unexpected command: $1" >&2; exit 65 ;;
esac
"#,
                shell_quote(&calls),
                shell_quote(&brief_path),
                shell_quote(&status_path)
            ),
        )
        .unwrap();
        make_executable(&fake_hk);
        let output = Command::new(python3())
            .arg(example_hk_provider())
            .arg("--repo")
            .arg(repo)
            .env_clear()
            .env("HOME", &home)
            .env("USER", "foreman-test")
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "provider failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let response: ProviderResponse = serde_json::from_slice(&output.stdout).unwrap();
        let calls = std::fs::read_to_string(&calls)
            .unwrap_or_default()
            .lines()
            .map(str::to_owned)
            .collect();
        (response, calls)
    }

    fn action_labels(card: &ControlExtensionCard) -> Vec<&str> {
        card.actions
            .iter()
            .map(|action| action.label.as_str())
            .collect()
    }

    fn row_value<'a>(card: &'a ControlExtensionCard, label: &str) -> Option<&'a str> {
        card.rows
            .iter()
            .find(|row| row.label == label)
            .map(|row| row.value.as_str())
    }

    fn shell_quote(path: &Path) -> String {
        format!("'{}'", path_string(path).replace('\'', "'\\''"))
    }

    #[test]
    fn explicit_provider_env_overrides_repo_provider_with_same_id() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let _home_guard = EnvVarGuard::set("HOME", home.as_os_str());
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let repo_providers_dir = temp.path().join(".foreman/providers");
        let env_providers_dir = temp.path().join("env-providers");
        std::fs::create_dir_all(&repo_providers_dir).unwrap();
        std::fs::create_dir_all(&env_providers_dir).unwrap();
        write_demo_provider(&repo_providers_dir, "repo");
        write_demo_provider(&env_providers_dir, "env");
        let _guard = EnvVarGuard::set(PROVIDER_ENV, env_providers_dir.as_os_str());

        let cards = collect_workspace_extensions(&[temp.path().to_path_buf()]);
        let cards = cards.get(temp.path()).unwrap();

        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].summary, "env");
    }

    #[test]
    fn workspace_scope_runs_once_per_workspace_path() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _provider_guard = EnvVarGuard::remove(PROVIDER_ENV);
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("home");
        std::fs::create_dir(&home).unwrap();
        let _home_guard = EnvVarGuard::set("HOME", home.as_os_str());
        std::fs::create_dir(temp.path().join(".git")).unwrap();
        let providers_dir = temp.path().join(".foreman/providers");
        std::fs::create_dir_all(&providers_dir).unwrap();
        let first = temp.path().join("packages/first");
        let second = temp.path().join("packages/second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        let script = providers_dir.join("workspace-provider.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf '%s\\n' '{\"cards\":[{\"id\":\"workspace\",\"title\":\"Workspace\",\"status\":\"ready\",\"statusLabel\":\"READY\",\"summary\":\"'\"$1\"'\"}]}'\n",
        )
        .unwrap();
        make_executable(&script);
        std::fs::write(
            providers_dir.join("workspace.toml"),
            "id = \"workspace\"\ntitle = \"Workspace\"\nscope = \"workspace\"\ncommand = \"./workspace-provider.sh\"\nargs = [\"{workspace}\"]\n",
        )
        .unwrap();

        let cards = collect_workspace_extensions(&[first.clone(), second.clone()]);

        assert_eq!(cards.get(&first).unwrap()[0].summary, path_string(&first));
        assert_eq!(cards.get(&second).unwrap()[0].summary, path_string(&second));
    }

    fn example_hk_provider() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/providers/harness-kit/hk-provider.py")
    }

    fn python3() -> PathBuf {
        if let Some(path) = std::env::var_os("PYTHON") {
            return PathBuf::from(path);
        }
        for candidate in [
            "/usr/bin/python3",
            "/opt/homebrew/bin/python3",
            "/usr/local/bin/python3",
        ] {
            let path = PathBuf::from(candidate);
            if path.is_file() {
                return path;
            }
        }
        PathBuf::from("python3")
    }

    fn write_demo_provider(dir: &Path, summary: &str) {
        let script = dir.join("provider.sh");
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nprintf '%s\\n' '{{\"cards\":[{{\"id\":\"demo\",\"title\":\"Demo\",\"status\":\"ready\",\"statusLabel\":\"READY\",\"summary\":\"{summary}\"}}]}}'\n"
            ),
        )
        .unwrap();
        make_executable(&script);
        std::fs::write(
            dir.join("demo.toml"),
            "id = \"demo\"\ntitle = \"Demo\"\ncommand = \"./provider.sh\"\n",
        )
        .unwrap();
    }

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(path, permissions).unwrap();
        }
    }
}
