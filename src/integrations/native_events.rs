use crate::app::AgentStatus;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeEventKind {
    RunStarted,
    RunFinished,
    ProcessExited,
    StatusChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct NativeEvent {
    pub schema_version: u16,
    pub source: String,
    pub pane_id: String,
    pub kind: NativeEventKind,
    pub occurred_at_unix_ms: u64,
    #[serde(default)]
    pub sequence: u64,
    #[serde(default)]
    pub run_id: Option<String>,
    #[serde(default)]
    pub process_id: Option<String>,
    #[serde(default)]
    pub status: Option<AgentStatus>,
}

impl NativeEvent {
    pub fn run_started(
        source: impl Into<String>,
        pane_id: impl Into<String>,
        run_id: impl Into<String>,
        process_id: impl Into<String>,
        occurred_at_unix_ms: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            source: source.into(),
            pane_id: pane_id.into(),
            kind: NativeEventKind::RunStarted,
            occurred_at_unix_ms,
            sequence: 0,
            run_id: Some(run_id.into()),
            process_id: Some(process_id.into()),
            status: None,
        }
    }

    pub fn run_finished(
        source: impl Into<String>,
        pane_id: impl Into<String>,
        run_id: impl Into<String>,
        process_id: impl Into<String>,
        occurred_at_unix_ms: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            source: source.into(),
            pane_id: pane_id.into(),
            kind: NativeEventKind::RunFinished,
            occurred_at_unix_ms,
            sequence: 0,
            run_id: Some(run_id.into()),
            process_id: Some(process_id.into()),
            status: None,
        }
    }

    pub fn process_exited(
        source: impl Into<String>,
        pane_id: impl Into<String>,
        process_id: impl Into<String>,
        occurred_at_unix_ms: u64,
    ) -> Self {
        Self {
            schema_version: 1,
            source: source.into(),
            pane_id: pane_id.into(),
            kind: NativeEventKind::ProcessExited,
            occurred_at_unix_ms,
            sequence: 0,
            run_id: None,
            process_id: Some(process_id.into()),
            status: None,
        }
    }

    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = sequence;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDerivedState {
    pub status: AgentStatus,
    pub active_run_count: u32,
    pub active_processes: BTreeMap<String, u64>,
    pub last_activity_unix_ms: Option<u64>,
    pub last_status_change_unix_ms: Option<u64>,
}

impl Default for NativeDerivedState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            active_run_count: 0,
            active_processes: BTreeMap::new(),
            last_activity_unix_ms: None,
            last_status_change_unix_ms: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ActiveRun {
    process_id: Option<String>,
    started_at_unix_ms: u64,
}

pub fn derive_native_state(events: &[NativeEvent]) -> NativeDerivedState {
    let mut ordered = events.to_vec();
    ordered.sort_by(|left, right| {
        left.occurred_at_unix_ms
            .cmp(&right.occurred_at_unix_ms)
            .then_with(|| left.sequence.cmp(&right.sequence))
    });

    let mut state = NativeDerivedState::default();
    let mut active_runs = BTreeMap::<String, ActiveRun>::new();

    for event in ordered {
        state.last_activity_unix_ms = Some(event.occurred_at_unix_ms);
        let previous_status = state.status;

        match event.kind {
            NativeEventKind::RunStarted => {
                if let Some(run_id) = event.run_id.as_ref() {
                    active_runs.insert(
                        run_id.clone(),
                        ActiveRun {
                            process_id: event.process_id.clone(),
                            started_at_unix_ms: event.occurred_at_unix_ms,
                        },
                    );
                }
            }
            NativeEventKind::RunFinished => {
                if let Some(run_id) = event.run_id.as_ref() {
                    active_runs.remove(run_id);
                }
            }
            NativeEventKind::ProcessExited => {
                if let Some(process_id) = event.process_id.as_ref() {
                    active_runs.retain(|_, run| run.process_id.as_deref() != Some(process_id));
                } else if let Some(run_id) = event.run_id.as_ref() {
                    active_runs.remove(run_id);
                }
            }
            NativeEventKind::StatusChanged => {}
        }

        state.active_run_count = active_runs.len() as u32;
        state.active_processes = active_runs
            .values()
            .filter_map(|run| {
                run.process_id
                    .as_ref()
                    .map(|process_id| (process_id.clone(), run.started_at_unix_ms))
            })
            .fold(
                BTreeMap::new(),
                |mut processes, (process_id, started_at)| {
                    processes
                        .entry(process_id)
                        .and_modify(|current| *current = (*current).min(started_at))
                        .or_insert(started_at);
                    processes
                },
            );
        let derived_status = if active_runs.is_empty() {
            AgentStatus::Idle
        } else {
            AgentStatus::Working
        };
        state.status = event.status.unwrap_or(derived_status);

        if state.status != previous_status {
            state.last_status_change_unix_ms = Some(event.occurred_at_unix_ms);
        }
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reducer_keeps_pane_working_until_every_overlapping_run_finishes() {
        let events = vec![
            NativeEvent::run_started("pi", "%1", "parent", "p0", 100).with_sequence(1),
            NativeEvent::run_started("pi", "%1", "child-a", "p1", 110).with_sequence(2),
            NativeEvent::run_started("pi", "%1", "child-b", "p2", 120).with_sequence(3),
            NativeEvent::run_finished("pi", "%1", "child-a", "p1", 130).with_sequence(4),
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Working);
        assert_eq!(state.active_run_count, 2);
        assert_eq!(state.last_activity_unix_ms, Some(130));
        assert_eq!(state.last_status_change_unix_ms, Some(100));
    }

    #[test]
    fn reducer_marks_idle_only_after_final_run_finishes() {
        let events = vec![
            NativeEvent::run_started("pi", "%1", "parent", "p0", 100).with_sequence(1),
            NativeEvent::run_started("pi", "%1", "child", "p1", 110).with_sequence(2),
            NativeEvent::run_finished("pi", "%1", "child", "p1", 120).with_sequence(3),
            NativeEvent::run_finished("pi", "%1", "parent", "p0", 130).with_sequence(4),
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.active_run_count, 0);
        assert_eq!(state.last_activity_unix_ms, Some(130));
        assert_eq!(state.last_status_change_unix_ms, Some(130));
    }

    #[test]
    fn process_exit_clears_only_matching_process_runs() {
        let events = vec![
            NativeEvent::run_started("pi", "%1", "first", "p1", 100).with_sequence(1),
            NativeEvent::run_started("pi", "%1", "second", "p2", 110).with_sequence(2),
            NativeEvent::process_exited("pi", "%1", "p1", 120).with_sequence(3),
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Working);
        assert_eq!(state.active_run_count, 1);
    }

    #[test]
    fn process_exit_with_only_run_id_clears_that_run() {
        let mut shutdown = NativeEvent::process_exited("pi", "%1", "unused", 120).with_sequence(2);
        shutdown.process_id = None;
        shutdown.run_id = Some("run".to_string());
        let events = vec![
            NativeEvent::run_started("pi", "%1", "run", "p1", 100).with_sequence(1),
            shutdown,
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.active_run_count, 0);
    }

    #[test]
    fn duplicate_start_and_finish_events_are_idempotent() {
        let events = vec![
            NativeEvent::run_started("pi", "%1", "run", "p1", 100).with_sequence(1),
            NativeEvent::run_started("pi", "%1", "run", "p1", 101).with_sequence(2),
            NativeEvent::run_finished("pi", "%1", "run", "p1", 102).with_sequence(3),
            NativeEvent::run_finished("pi", "%1", "run", "p1", 103).with_sequence(4),
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.active_run_count, 0);
    }

    #[test]
    fn reducer_orders_by_timestamp_then_sequence() {
        let events = vec![
            NativeEvent::run_finished("pi", "%1", "run", "p1", 100).with_sequence(2),
            NativeEvent::run_started("pi", "%1", "run", "p1", 100).with_sequence(1),
        ];

        let state = derive_native_state(&events);

        assert_eq!(state.status, AgentStatus::Idle);
        assert_eq!(state.last_activity_unix_ms, Some(100));
        assert_eq!(state.last_status_change_unix_ms, Some(100));
    }
}
