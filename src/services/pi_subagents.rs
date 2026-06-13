use crate::app::HarnessKind;
use crate::services::extensions::{ControlExtensionCard, ControlExtensionRow};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AsyncStatus {
    run_id: String,
    state: String,
    #[serde(default)]
    activity_state: Option<String>,
    #[serde(default)]
    current_tool: Option<String>,
    #[serde(default)]
    current_path: Option<String>,
    #[serde(default)]
    turn_count: Option<u64>,
    #[serde(default)]
    tool_count: Option<u64>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    steps: Vec<AsyncStepStatus>,
    #[serde(default)]
    nested_children: Vec<NestedRunSummary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AsyncStepStatus {
    agent: String,
    status: String,
    #[serde(default)]
    activity_state: Option<String>,
    #[serde(default)]
    current_tool: Option<String>,
    #[serde(default)]
    current_path: Option<String>,
    #[serde(default)]
    children: Vec<NestedRunSummary>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NestedRunSummary {
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    activity_state: Option<String>,
}

pub fn collect_pi_subagent_cards(
    harness: Option<HarnessKind>,
    working_dir: Option<&Path>,
) -> Vec<ControlExtensionCard> {
    if harness != Some(HarnessKind::Pi) {
        return Vec::new();
    }
    let Some(working_dir) = working_dir else {
        return Vec::new();
    };

    let runs = active_runs_for_workspace(&default_async_dir(), working_dir);
    if runs.is_empty() {
        return Vec::new();
    }

    vec![card_for_runs(&runs)]
}

fn active_runs_for_workspace(async_dir: &Path, working_dir: &Path) -> Vec<AsyncStatus> {
    let Ok(entries) = fs::read_dir(async_dir) else {
        return Vec::new();
    };
    let mut runs = entries
        .filter_map(Result::ok)
        .filter_map(|entry| read_status_file(&entry.path().join("status.json")))
        .filter(|status| is_active_state(&status.state))
        .filter(|status| {
            status
                .cwd
                .as_deref()
                .is_some_and(|cwd| workspace_matches(cwd, working_dir))
        })
        .collect::<Vec<_>>();
    runs.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    runs
}

fn read_status_file(path: &Path) -> Option<AsyncStatus> {
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn is_active_state(state: &str) -> bool {
    matches!(state, "queued" | "running" | "failed" | "paused")
}

fn workspace_matches(run_cwd: &Path, working_dir: &Path) -> bool {
    run_cwd == working_dir || run_cwd.starts_with(working_dir) || working_dir.starts_with(run_cwd)
}

fn card_for_runs(runs: &[AsyncStatus]) -> ControlExtensionCard {
    let mut counts = BTreeMap::<&str, usize>::new();
    for run in runs {
        *counts.entry(run.state.as_str()).or_default() += 1;
    }
    let needs_attention = runs.iter().any(run_needs_attention);
    let status = if needs_attention {
        "needs-attention"
    } else if counts.contains_key("failed") {
        "error"
    } else {
        "working"
    };
    let status_label = match status {
        "needs-attention" => "NEEDS ATTENTION",
        "error" => "FAILED",
        _ => "ACTIVE",
    };

    let primary = &runs[0];
    let summary = format_counts(&counts);
    let mut rows = vec![
        ControlExtensionRow {
            label: "Runs".to_string(),
            value: summary.clone(),
            status: Some(status.to_string()),
        },
        ControlExtensionRow {
            label: "Current".to_string(),
            value: current_activity(primary),
            status: Some(row_status(primary)),
        },
    ];

    if runs.len() > 1 {
        rows.push(ControlExtensionRow {
            label: "Run ids".to_string(),
            value: runs
                .iter()
                .take(3)
                .map(|run| run.run_id.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            status: Some("info".to_string()),
        });
    }

    ControlExtensionCard {
        id: "pi-subagents".to_string(),
        title: "Pi subagents".to_string(),
        status: status.to_string(),
        status_label: status_label.to_string(),
        summary,
        rows,
        actions: Vec::new(),
    }
}

fn format_counts(counts: &BTreeMap<&str, usize>) -> String {
    ["running", "queued", "failed", "paused"]
        .into_iter()
        .filter_map(|state| counts.get(state).map(|count| format!("{count} {state}")))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn current_activity(run: &AsyncStatus) -> String {
    if let Some(tool) = run.current_tool.as_deref() {
        return append_path(format!("tool {tool}"), run.current_path.as_deref());
    }
    for step in &run.steps {
        if step.status == "running" {
            if let Some(tool) = step.current_tool.as_deref() {
                return append_path(
                    format!("{}: {tool}", step.agent),
                    step.current_path.as_deref(),
                );
            }
            return step.agent.clone();
        }
    }
    let nested = run
        .nested_children
        .iter()
        .chain(run.steps.iter().flat_map(|step| step.children.iter()))
        .filter(|child| child.state.as_deref().is_some_and(is_active_state))
        .count();
    if nested > 0 {
        return format!("{nested} nested active");
    }
    match (run.turn_count, run.tool_count) {
        (Some(turns), Some(tools)) => format!("{turns} turns · {tools} tools"),
        (Some(turns), None) => format!("{turns} turns"),
        (None, Some(tools)) => format!("{tools} tools"),
        (None, None) => run.run_id.clone(),
    }
}

fn append_path(mut label: String, path: Option<&str>) -> String {
    if let Some(path) = path {
        label.push_str(" · ");
        label.push_str(path);
    }
    label
}

fn row_status(run: &AsyncStatus) -> String {
    if run_needs_attention(run) {
        "needs-attention".to_string()
    } else if run.state == "failed" {
        "fail".to_string()
    } else {
        "working".to_string()
    }
}

fn run_needs_attention(run: &AsyncStatus) -> bool {
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

fn default_async_dir() -> PathBuf {
    std::env::temp_dir()
        .join(format!("pi-subagents-{}", temp_scope_id()))
        .join("async-subagent-runs")
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
    fn active_runs_are_filtered_by_workspace() {
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

        let runs = active_runs_for_workspace(temp.path(), &matching);

        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].run_id, "run-a");
    }

    #[test]
    fn card_marks_attention_when_any_step_needs_attention() {
        let runs = vec![AsyncStatus {
            run_id: "run".to_string(),
            state: "running".to_string(),
            activity_state: None,
            current_tool: None,
            current_path: None,
            turn_count: Some(2),
            tool_count: Some(4),
            cwd: None,
            steps: vec![AsyncStepStatus {
                agent: "reviewer".to_string(),
                status: "running".to_string(),
                activity_state: Some("needs_attention".to_string()),
                current_tool: Some("Bash".to_string()),
                current_path: None,
                children: Vec::new(),
            }],
            nested_children: Vec::new(),
        }];

        let card = card_for_runs(&runs);

        assert_eq!(card.status, "needs-attention");
        assert_eq!(card.rows[1].value, "reviewer: Bash");
    }

    fn write_status(root: &Path, run_id: &str, json: &str) {
        let dir = root.join(run_id);
        fs::create_dir_all(&dir).expect("run dir");
        fs::write(dir.join("status.json"), json).expect("status");
    }
}
