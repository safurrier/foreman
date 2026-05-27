use crate::app::{AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory, PaneId};
use crate::integrations::recognize_pane_runtime_identity;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSignal {
    pub status: AgentStatus,
    pub activity_score: u64,
    pub recent_activity_unix_millis: Option<u64>,
    pub active_run_count: Option<u32>,
    pub last_activity_unix_millis: Option<u64>,
    pub last_status_change_unix_millis: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NativeOverlaySummary {
    pub applied: usize,
    pub fallback_to_compatibility: usize,
    pub warnings: Vec<String>,
}

pub trait NativeSignalSource {
    fn signal_for_pane(&self, pane_id: &PaneId) -> Result<Option<NativeSignal>, NativeSignalError>;
}

#[derive(Debug)]
pub enum NativeSignalError {
    Io(io::Error),
    Parse(serde_json::Error),
    InvalidStatus(String),
}

impl fmt::Display for NativeSignalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
            Self::InvalidStatus(status) => write!(f, "invalid native status: {status}"),
        }
    }
}

impl std::error::Error for NativeSignalError {}

impl From<io::Error> for NativeSignalError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for NativeSignalError {
    fn from(error: serde_json::Error) -> Self {
        Self::Parse(error)
    }
}

#[derive(Debug)]
pub enum NativeSignalWriteError {
    Io(io::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for NativeSignalWriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Serialize(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for NativeSignalWriteError {}

impl From<io::Error> for NativeSignalWriteError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for NativeSignalWriteError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialize(error)
    }
}

#[derive(Debug, Clone)]
pub struct FileNativeSignalSource {
    dir: PathBuf,
}

impl FileNativeSignalSource {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn signal_path(&self, pane_id: &PaneId) -> PathBuf {
        self.dir.join(format!("{}.json", pane_id.as_str()))
    }
}

impl NativeSignalSource for FileNativeSignalSource {
    fn signal_for_pane(&self, pane_id: &PaneId) -> Result<Option<NativeSignal>, NativeSignalError> {
        let path = self.signal_path(pane_id);
        if !path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&path).ok();
        let recent_activity_unix_millis = metadata
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as u64);
        let payload = fs::read_to_string(path)?;
        let raw: RawNativeSignal = serde_json::from_str(&payload)?;
        let status = parse_native_status(&raw.status)?;
        let last_activity_unix_millis = raw.last_activity_unix_millis;
        Ok(Some(NativeSignal {
            status,
            activity_score: raw
                .activity_score
                .unwrap_or_else(|| native_activity_score(status)),
            recent_activity_unix_millis: last_activity_unix_millis.or(recent_activity_unix_millis),
            active_run_count: raw.active_run_count,
            last_activity_unix_millis,
            last_status_change_unix_millis: raw.last_status_change_unix_millis,
        }))
    }
}

#[derive(Debug, Deserialize)]
struct RawNativeSignal {
    status: String,
    activity_score: Option<u64>,
    active_run_count: Option<u32>,
    last_activity_unix_millis: Option<u64>,
    last_status_change_unix_millis: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SignalFile<'a> {
    status: &'a str,
    activity_score: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_version: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_rank: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_run_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_activity_unix_millis: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_status_change_unix_millis: Option<u64>,
}

pub fn signal_for_status(status: AgentStatus) -> NativeSignal {
    NativeSignal {
        status,
        activity_score: native_activity_score(status),
        recent_activity_unix_millis: None,
        active_run_count: None,
        last_activity_unix_millis: None,
        last_status_change_unix_millis: None,
    }
}

pub fn write_signal_file(
    native_dir: &Path,
    pane_id: &str,
    signal: &NativeSignal,
) -> Result<(), NativeSignalWriteError> {
    fs::create_dir_all(native_dir)?;

    let final_path = native_dir.join(format!("{pane_id}.json"));
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let temp_path = native_dir.join(format!(".{pane_id}.{unique_suffix}.tmp"));
    let has_v2_fields = signal.active_run_count.is_some()
        || signal.last_activity_unix_millis.is_some()
        || signal.last_status_change_unix_millis.is_some();
    let payload = SignalFile {
        status: status_label(signal.status),
        activity_score: signal.activity_score,
        schema_version: has_v2_fields.then_some(2),
        status_rank: has_v2_fields.then_some(status_rank(signal.status)),
        active_run_count: signal.active_run_count,
        last_activity_unix_millis: signal.last_activity_unix_millis,
        last_status_change_unix_millis: signal.last_status_change_unix_millis,
    };
    let contents = serde_json::to_vec(&payload)?;
    fs::write(&temp_path, contents)?;

    if let Err(error) = fs::rename(&temp_path, &final_path) {
        if final_path.exists() {
            fs::remove_file(&final_path)?;
            fs::rename(&temp_path, &final_path)?;
        } else {
            return Err(NativeSignalWriteError::Io(error));
        }
    }

    Ok(())
}

pub fn apply_native_signals<S: NativeSignalSource>(
    inventory: &mut Inventory,
    harness: HarnessKind,
    source: &S,
) -> NativeOverlaySummary {
    let mut summary = NativeOverlaySummary::default();

    for session in &mut inventory.sessions {
        for window in &mut session.windows {
            for pane in &mut window.panes {
                let runtime_identity = recognize_pane_runtime_identity(
                    pane.current_command.as_deref(),
                    pane.runtime_command.as_deref(),
                    &pane.title,
                );
                let native_candidate = match runtime_identity {
                    Some(identity) => identity == harness,
                    None => matches!(
                        pane.agent.as_ref(),
                        Some(agent) if agent.integration_mode == IntegrationMode::Compatibility
                    ),
                };
                let fallback_candidate = matches!(
                    pane.agent.as_ref(),
                    Some(agent)
                        if agent.harness == harness
                            && agent.integration_mode == IntegrationMode::Compatibility
                );

                match source.signal_for_pane(&pane.id) {
                    Ok(Some(signal)) if native_candidate => {
                        pane.agent = Some(AgentSnapshot {
                            harness,
                            status: signal.status,
                            observed_status: signal.status,
                            integration_mode: IntegrationMode::Native,
                            activity_score: signal.activity_score,
                            debounce_ticks: 0,
                            active_run_count: signal.active_run_count,
                            last_status_change_unix_millis: signal.last_status_change_unix_millis,
                        });
                        pane.activity_unix_millis = signal
                            .recent_activity_unix_millis
                            .or(pane.activity_unix_millis);
                        summary.applied += 1;
                    }
                    Ok(Some(_)) => {
                        summary.warnings.push(format!(
                            "pane={} ignored stale {} native signal; current_command={} runtime_command={} title={}",
                            pane.id.as_str(),
                            harness.filter_label(),
                            diagnostic_value(pane.current_command.as_deref()),
                            diagnostic_value(pane.runtime_command.as_deref()),
                            diagnostic_value(Some(&pane.title)),
                        ));
                    }
                    Ok(None) => {
                        if fallback_candidate {
                            summary.fallback_to_compatibility += 1;
                        }
                    }
                    Err(error) => {
                        if fallback_candidate {
                            summary.fallback_to_compatibility += 1;
                        }
                        summary
                            .warnings
                            .push(format!("pane={} {error}", pane.id.as_str()));
                    }
                }
            }
        }
    }

    summary
}

pub fn compatibility_fallback_summary(
    inventory: &Inventory,
    harness: HarnessKind,
    warning: Option<String>,
) -> NativeOverlaySummary {
    let fallback_to_compatibility = inventory
        .sessions
        .iter()
        .flat_map(|session| session.windows.iter())
        .flat_map(|window| window.panes.iter())
        .filter(|pane| {
            matches!(
                pane.agent.as_ref(),
                Some(agent)
                    if agent.harness == harness
                        && agent.integration_mode == IntegrationMode::Compatibility
            )
        })
        .count();

    let mut summary = NativeOverlaySummary {
        applied: 0,
        fallback_to_compatibility,
        warnings: Vec::new(),
    };
    if let Some(warning) = warning.filter(|_| fallback_to_compatibility > 0) {
        summary.warnings.push(warning);
    }

    summary
}

fn diagnostic_value(value: Option<&str>) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.replace('\n', "\\n"))
        .unwrap_or_else(|| "<none>".to_string())
}

fn parse_native_status(status: &str) -> Result<AgentStatus, NativeSignalError> {
    match status.trim().to_ascii_lowercase().as_str() {
        "working" => Ok(AgentStatus::Working),
        "needs_attention" | "needs-attention" | "attention" => Ok(AgentStatus::NeedsAttention),
        "idle" => Ok(AgentStatus::Idle),
        "error" => Ok(AgentStatus::Error),
        "unknown" => Ok(AgentStatus::Unknown),
        other => Err(NativeSignalError::InvalidStatus(other.to_string())),
    }
}

fn native_activity_score(status: AgentStatus) -> u64 {
    match status {
        AgentStatus::Working => 120,
        AgentStatus::NeedsAttention => 90,
        AgentStatus::Error => 70,
        AgentStatus::Idle => 40,
        AgentStatus::Unknown => 0,
    }
}

pub fn status_rank(status: AgentStatus) -> u8 {
    status.attention_rank()
}

fn status_label(status: AgentStatus) -> &'static str {
    match status {
        AgentStatus::Working => "working",
        AgentStatus::NeedsAttention => "needs_attention",
        AgentStatus::Error => "error",
        AgentStatus::Idle => "idle",
        AgentStatus::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        apply_native_signals, compatibility_fallback_summary, signal_for_status, write_signal_file,
        FileNativeSignalSource, NativeOverlaySummary, NativeSignal, NativeSignalSource,
    };
    use crate::app::{
        inventory, AgentStatus, HarnessKind, IntegrationMode, PaneBuilder, SessionBuilder,
        WindowBuilder,
    };
    use tempfile::tempdir;

    #[test]
    fn file_signal_source_reads_native_signal() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%7.json"),
            r#"{"status":"needs_attention","activity_score":95}"#,
        )
        .expect("signal should exist");

        let signal = FileNativeSignalSource::new(temp_dir.path().to_path_buf())
            .signal_for_pane(&"%7".into())
            .expect("signal read should succeed")
            .expect("signal should exist");

        assert_eq!(signal.status, AgentStatus::NeedsAttention);
        assert_eq!(signal.activity_score, 95);
    }

    #[test]
    fn write_signal_file_writes_atomic_payload() {
        let temp_dir = tempdir().expect("temp dir should exist");
        write_signal_file(
            temp_dir.path(),
            "%9",
            &signal_for_status(AgentStatus::Working),
        )
        .expect("write should succeed");

        let contents =
            std::fs::read_to_string(temp_dir.path().join("%9.json")).expect("signal should exist");
        assert_eq!(contents, r#"{"status":"working","activity_score":120}"#);
    }

    #[test]
    fn v2_signal_file_round_trips_explicit_activity_fields() {
        let temp_dir = tempdir().expect("temp dir should exist");
        write_signal_file(
            temp_dir.path(),
            "%9",
            &NativeSignal {
                status: AgentStatus::Working,
                activity_score: 120,
                recent_activity_unix_millis: Some(200),
                active_run_count: Some(2),
                last_activity_unix_millis: Some(200),
                last_status_change_unix_millis: Some(100),
            },
        )
        .expect("write should succeed");

        let signal = FileNativeSignalSource::new(temp_dir.path().to_path_buf())
            .signal_for_pane(&"%9".into())
            .expect("signal read should succeed")
            .expect("signal should exist");

        assert_eq!(signal.status, AgentStatus::Working);
        assert_eq!(signal.active_run_count, Some(2));
        assert_eq!(signal.recent_activity_unix_millis, Some(200));
        assert_eq!(signal.last_activity_unix_millis, Some(200));
        assert_eq!(signal.last_status_change_unix_millis, Some(100));
    }

    #[test]
    fn apply_native_signals_overlays_matching_harness() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%2.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal should exist");
        let source = FileNativeSignalSource::new(temp_dir.path().to_path_buf());
        let mut inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1")
                .name("main")
                .pane(
                    PaneBuilder::agent("%2", HarnessKind::ClaudeCode)
                        .title("claude")
                        .status(AgentStatus::Working),
                )
                .pane(
                    PaneBuilder::agent("%3", HarnessKind::CodexCli)
                        .title("codex")
                        .status(AgentStatus::Working),
                ),
        )]);

        let summary = apply_native_signals(&mut inventory, HarnessKind::ClaudeCode, &source);

        let claude = inventory
            .pane(&"%2".into())
            .and_then(|pane| pane.agent.as_ref())
            .expect("claude pane should exist");
        assert_eq!(claude.integration_mode, IntegrationMode::Native);
        assert_eq!(claude.status, AgentStatus::Idle);

        let codex = inventory
            .pane(&"%3".into())
            .and_then(|pane| pane.agent.as_ref())
            .expect("codex pane should exist");
        assert_eq!(codex.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn apply_native_signals_does_not_override_different_harness() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%4.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal should exist");
        let source = FileNativeSignalSource::new(temp_dir.path().to_path_buf());
        let mut inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1").name("main").pane(
                PaneBuilder::agent("%4", HarnessKind::Pi)
                    .title("π - pi")
                    .status(AgentStatus::Working),
            ),
        )]);

        let summary = apply_native_signals(&mut inventory, HarnessKind::CodexCli, &source);

        let agent = inventory
            .pane(&"%4".into())
            .and_then(|pane| pane.agent.as_ref())
            .expect("pane should remain an agent");
        assert_eq!(agent.harness, HarnessKind::Pi);
        assert_eq!(agent.integration_mode, IntegrationMode::Compatibility);
        assert_eq!(agent.status, AgentStatus::Working);
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.fallback_to_compatibility, 0);
    }

    #[test]
    fn apply_native_signals_ignores_file_only_shell_pane_candidates() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%5.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal should exist");
        let source = FileNativeSignalSource::new(temp_dir.path().to_path_buf());
        let mut inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1").name("main").pane(
                PaneBuilder::new("%5")
                    .title("plain shell")
                    .current_command("zsh")
                    .runtime_command("zsh"),
            ),
        )]);

        let summary = apply_native_signals(&mut inventory, HarnessKind::CodexCli, &source);

        assert!(inventory
            .pane(&"%5".into())
            .expect("pane should exist")
            .agent
            .is_none());
        assert_eq!(summary.applied, 0);
        assert_eq!(summary.fallback_to_compatibility, 0);
        assert_eq!(summary.warnings.len(), 1);
        assert!(summary.warnings[0].contains("ignored stale codex native signal"));
    }

    #[test]
    fn apply_native_signals_can_correct_compatibility_mislabel_without_runtime_identity() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%7.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal should exist");
        let source = FileNativeSignalSource::new(temp_dir.path().to_path_buf());
        let mut inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1").name("main").pane(
                PaneBuilder::agent("%7", HarnessKind::ClaudeCode)
                    .title("stale claude text")
                    .status(AgentStatus::Working)
                    .integration_mode(IntegrationMode::Compatibility),
            ),
        )]);

        let summary = apply_native_signals(&mut inventory, HarnessKind::CodexCli, &source);

        let agent = inventory
            .pane(&"%7".into())
            .and_then(|pane| pane.agent.as_ref())
            .expect("pane should become a native agent");
        assert_eq!(agent.harness, HarnessKind::CodexCli);
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.warnings.len(), 0);
    }

    #[test]
    fn apply_native_signals_still_overlays_runtime_identity_without_compatibility_snapshot() {
        let temp_dir = tempdir().expect("temp dir should exist");
        std::fs::write(
            temp_dir.path().join("%6.json"),
            r#"{"status":"idle","activity_score":44}"#,
        )
        .expect("signal should exist");
        let source = FileNativeSignalSource::new(temp_dir.path().to_path_buf());
        let mut inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1").name("main").pane(
                PaneBuilder::new("%6")
                    .title("codex")
                    .current_command("node")
                    .runtime_command("npm exec @openai/codex --model gpt-5"),
            ),
        )]);

        let summary = apply_native_signals(&mut inventory, HarnessKind::CodexCli, &source);

        let agent = inventory
            .pane(&"%6".into())
            .and_then(|pane| pane.agent.as_ref())
            .expect("pane should become a native agent");
        assert_eq!(agent.harness, HarnessKind::CodexCli);
        assert_eq!(agent.integration_mode, IntegrationMode::Native);
        assert_eq!(agent.status, AgentStatus::Idle);
        assert_eq!(summary.applied, 1);
        assert_eq!(summary.warnings.len(), 0);
    }

    #[test]
    fn compatibility_fallback_summary_counts_matching_harness_only() {
        let inventory = inventory(vec![SessionBuilder::new("s1").window(
            WindowBuilder::new("w1")
                .name("main")
                .pane(
                    PaneBuilder::agent("%2", HarnessKind::ClaudeCode)
                        .title("claude")
                        .status(AgentStatus::Working),
                )
                .pane(
                    PaneBuilder::agent("%3", HarnessKind::CodexCli)
                        .title("codex")
                        .status(AgentStatus::Working),
                ),
        )]);

        let summary = compatibility_fallback_summary(
            &inventory,
            HarnessKind::CodexCli,
            Some("missing".to_string()),
        );

        assert_eq!(
            summary,
            NativeOverlaySummary {
                applied: 0,
                fallback_to_compatibility: 1,
                warnings: vec!["missing".to_string()],
            }
        );
    }
}
