use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const DEFAULT_ACTIVITY_TTL_MS: u64 = 5 * 60 * 1_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PiSubagentActivity {
    pub(crate) active_run_count: u32,
    pub(crate) failed_count: u32,
    pub(crate) needs_attention: bool,
    pub(crate) last_activity_unix_millis: Option<u64>,
    pub(crate) runs: Vec<AsyncStatus>,
}

impl PiSubagentActivity {
    pub(crate) fn has_active_work(&self) -> bool {
        self.active_run_count > 0
    }
}

pub(crate) trait PiSubagentActivitySource {
    fn activity_for_workspace(&self, working_dir: &Path) -> Option<PiSubagentActivity>;
}

#[derive(Debug, Clone)]
pub(crate) struct FilePiSubagentActivitySource {
    async_dir: PathBuf,
    max_age_ms: u64,
}

impl FilePiSubagentActivitySource {
    pub(crate) fn default_user() -> Self {
        Self {
            async_dir: default_async_dir(),
            max_age_ms: DEFAULT_ACTIVITY_TTL_MS,
        }
    }
}

impl PiSubagentActivitySource for FilePiSubagentActivitySource {
    fn activity_for_workspace(&self, working_dir: &Path) -> Option<PiSubagentActivity> {
        activity_for_workspace_at(
            &self.async_dir,
            working_dir,
            current_time_ms(),
            self.max_age_ms,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AsyncStatus {
    pub(crate) run_id: String,
    pub(crate) state: String,
    #[serde(default)]
    pub(crate) activity_state: Option<String>,
    #[serde(default)]
    pub(crate) current_tool: Option<String>,
    #[serde(default)]
    pub(crate) current_path: Option<String>,
    #[serde(default)]
    pub(crate) turn_count: Option<u64>,
    #[serde(default)]
    pub(crate) tool_count: Option<u64>,
    #[serde(default)]
    pub(crate) cwd: Option<PathBuf>,
    #[serde(default)]
    pub(crate) steps: Vec<AsyncStepStatus>,
    #[serde(default)]
    pub(crate) nested_children: Vec<NestedRunSummary>,
    #[serde(skip)]
    pub(crate) observed_unix_millis: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AsyncStepStatus {
    pub(crate) agent: String,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) activity_state: Option<String>,
    #[serde(default)]
    pub(crate) current_tool: Option<String>,
    #[serde(default)]
    pub(crate) current_path: Option<String>,
    #[serde(default)]
    pub(crate) children: Vec<NestedRunSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NestedRunSummary {
    #[serde(default)]
    pub(crate) state: Option<String>,
    #[serde(default)]
    pub(crate) activity_state: Option<String>,
}

pub(crate) fn activity_for_workspace(working_dir: &Path) -> Option<PiSubagentActivity> {
    FilePiSubagentActivitySource::default_user().activity_for_workspace(working_dir)
}

#[cfg(test)]
pub(crate) fn activity_for_workspace_at(
    async_dir: &Path,
    working_dir: &Path,
    now_unix_millis: u64,
    max_age_ms: u64,
) -> Option<PiSubagentActivity> {
    activity_for_workspace_at_impl(async_dir, working_dir, now_unix_millis, max_age_ms)
}

#[cfg(not(test))]
fn activity_for_workspace_at(
    async_dir: &Path,
    working_dir: &Path,
    now_unix_millis: u64,
    max_age_ms: u64,
) -> Option<PiSubagentActivity> {
    activity_for_workspace_at_impl(async_dir, working_dir, now_unix_millis, max_age_ms)
}

fn activity_for_workspace_at_impl(
    async_dir: &Path,
    working_dir: &Path,
    now_unix_millis: u64,
    max_age_ms: u64,
) -> Option<PiSubagentActivity> {
    let Ok(entries) = fs::read_dir(async_dir) else {
        return None;
    };
    let mut runs = entries
        .filter_map(Result::ok)
        .filter_map(|entry| read_status_file(&entry.path().join("status.json")))
        .filter(|status| is_relevant_state(&status.state))
        .filter(|status| is_fresh(status.observed_unix_millis, now_unix_millis, max_age_ms))
        .filter(|status| {
            status
                .cwd
                .as_deref()
                .is_some_and(|cwd| workspace_matches(cwd, working_dir))
        })
        .collect::<Vec<_>>();
    runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    activity_from_runs(runs)
}

fn activity_from_runs(runs: Vec<AsyncStatus>) -> Option<PiSubagentActivity> {
    if runs.is_empty() {
        return None;
    }

    let active_run_count = runs
        .iter()
        .filter(|run| is_active_work_state(&run.state))
        .count() as u32;
    let failed_count = runs.iter().filter(|run| run.state == "failed").count() as u32;
    let needs_attention = runs.iter().any(run_needs_attention);
    let last_activity_unix_millis = runs.iter().filter_map(|run| run.observed_unix_millis).max();

    Some(PiSubagentActivity {
        active_run_count,
        failed_count,
        needs_attention,
        last_activity_unix_millis,
        runs,
    })
}

fn read_status_file(path: &Path) -> Option<AsyncStatus> {
    let observed_unix_millis = status_file_observed_unix_millis(path);
    let contents = fs::read_to_string(path).ok()?;
    let mut status: AsyncStatus = serde_json::from_str(&contents).ok()?;
    status.observed_unix_millis = observed_unix_millis;
    Some(status)
}

fn status_file_observed_unix_millis(path: &Path) -> Option<u64> {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
}

fn is_relevant_state(state: &str) -> bool {
    is_active_work_state(state) || state == "failed"
}

pub(crate) fn is_active_work_state(state: &str) -> bool {
    matches!(state, "queued" | "running" | "paused")
}

fn is_fresh(observed_unix_millis: Option<u64>, now_unix_millis: u64, max_age_ms: u64) -> bool {
    observed_unix_millis.is_some_and(|observed| {
        observed <= now_unix_millis && now_unix_millis.saturating_sub(observed) <= max_age_ms
    })
}

pub(crate) fn workspace_matches(run_cwd: &Path, working_dir: &Path) -> bool {
    run_cwd == working_dir || run_cwd.starts_with(working_dir) || working_dir.starts_with(run_cwd)
}

pub(crate) fn run_needs_attention(run: &AsyncStatus) -> bool {
    run.activity_state.as_deref() == Some("needs_attention")
        || run
            .steps
            .iter()
            .any(|step| step.activity_state.as_deref() == Some("needs_attention"))
        || run
            .nested_children
            .iter()
            .chain(run.steps.iter().flat_map(|step| step.children.iter()))
            .any(|child| child.activity_state.as_deref() == Some("needs_attention"))
}

pub(crate) fn state_counts(runs: &[AsyncStatus]) -> BTreeMap<&str, usize> {
    let mut counts = BTreeMap::<&str, usize>::new();
    for run in runs {
        *counts.entry(run.state.as_str()).or_default() += 1;
    }
    counts
}

fn default_async_dir() -> PathBuf {
    std::env::temp_dir()
        .join(format!("pi-subagents-{}", temp_scope_id()))
        .join("async-subagent-runs")
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(unix)]
fn temp_scope_id() -> String {
    unsafe extern "C" {
        fn getuid() -> u32;
    }
    format!("uid-{}", unsafe { getuid() })
}

#[cfg(not(unix))]
fn temp_scope_id() -> String {
    for key in ["USERNAME", "USER", "LOGNAME"] {
        if let Ok(value) = std::env::var(key) {
            let sanitized = sanitize_temp_scope_segment(&value);
            if !sanitized.is_empty() {
                return format!("user-{sanitized}");
            }
        }
    }
    "shared".to_string()
}

#[cfg(not(unix))]
fn sanitize_temp_scope_segment(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn activity_filters_by_workspace_and_reports_counts() {
        let temp = tempdir().expect("temp dir");
        let matching = temp.path().join("matching");
        let other = temp.path().join("other");
        fs::create_dir_all(&matching).expect("matching dir");
        fs::create_dir_all(&other).expect("other dir");
        write_status(
            temp.path(),
            "run-a",
            &format!(
                r#"{{"runId":"run-a","state":"running","cwd":"{}","currentTool":"Edit"}}"#,
                matching.display()
            ),
        );
        write_status(
            temp.path(),
            "run-b",
            &format!(
                r#"{{"runId":"run-b","state":"running","cwd":"{}"}}"#,
                other.display()
            ),
        );
        let now = current_time_ms();

        let activity =
            activity_for_workspace_at(temp.path(), &matching, now, DEFAULT_ACTIVITY_TTL_MS)
                .expect("activity");

        assert_eq!(activity.active_run_count, 1);
        assert_eq!(activity.failed_count, 0);
        assert_eq!(activity.runs[0].run_id, "run-a");
    }

    #[test]
    fn stale_status_files_do_not_report_activity() {
        let temp = tempdir().expect("temp dir");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).expect("workspace dir");
        let status_path = write_status(
            temp.path(),
            "run-a",
            &format!(
                r#"{{"runId":"run-a","state":"running","cwd":"{}"}}"#,
                workspace.display()
            ),
        );
        let observed = status_file_observed_unix_millis(&status_path).expect("mtime");

        let activity =
            activity_for_workspace_at(temp.path(), &workspace, observed + 10_001, 10_000);

        assert!(activity.is_none());
    }

    #[test]
    fn failed_runs_are_evidence_but_not_active_work() {
        let temp = tempdir().expect("temp dir");
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(&workspace).expect("workspace dir");
        write_status(
            temp.path(),
            "run-a",
            &format!(
                r#"{{"runId":"run-a","state":"failed","cwd":"{}"}}"#,
                workspace.display()
            ),
        );

        let activity =
            activity_for_workspace_at(temp.path(), &workspace, current_time_ms(), 10_000)
                .expect("activity");

        assert_eq!(activity.active_run_count, 0);
        assert_eq!(activity.failed_count, 1);
        assert!(!activity.has_active_work());
    }

    fn write_status(root: &Path, run_id: &str, json: &str) -> PathBuf {
        let dir = root.join(run_id);
        fs::create_dir_all(&dir).expect("run dir");
        let path = dir.join("status.json");
        fs::write(&path, json).expect("status");
        path
    }
}
