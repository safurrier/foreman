use crate::app::HarnessKind;
use crate::integrations::pi_subagents::{
    activity_for_workspace, is_active_work_state, run_needs_attention, state_counts, AsyncStatus,
};
use crate::services::extensions::{ControlExtensionCard, ControlExtensionRow};
use std::collections::BTreeMap;
use std::path::Path;

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

    let Some(activity) = activity_for_workspace(working_dir) else {
        return Vec::new();
    };
    vec![card_for_runs(&activity.runs)]
}

fn card_for_runs(runs: &[AsyncStatus]) -> ControlExtensionCard {
    let counts = state_counts(runs);
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
        .filter(|child| child.state.as_deref().is_some_and(is_active_work_state))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::pi_subagents::AsyncStepStatus;

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
            observed_unix_millis: None,
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
}
