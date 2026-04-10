use crate::app::{AgentSnapshot, AgentStatus, HarnessKind, IntegrationMode, Inventory, PaneId};
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

        let payload = fs::read_to_string(path)?;
        let raw: RawNativeSignal = serde_json::from_str(&payload)?;
        let status = parse_native_status(&raw.status)?;
        Ok(Some(NativeSignal {
            status,
            activity_score: raw
                .activity_score
                .unwrap_or_else(|| native_activity_score(status)),
        }))
    }
}

#[derive(Debug, Deserialize)]
struct RawNativeSignal {
    status: String,
    activity_score: Option<u64>,
}

#[derive(Debug, Serialize)]
struct SignalFile<'a> {
    status: &'a str,
    activity_score: u64,
}

pub fn signal_for_status(status: AgentStatus) -> NativeSignal {
    NativeSignal {
        status,
        activity_score: native_activity_score(status),
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
    let payload = SignalFile {
        status: status_label(signal.status),
        activity_score: signal.activity_score,
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
                let fallback_candidate = matches!(
                    pane.agent.as_ref(),
                    Some(agent)
                        if agent.harness == harness
                            && agent.integration_mode == IntegrationMode::Compatibility
                );

                match source.signal_for_pane(&pane.id) {
                    Ok(Some(signal)) => {
                        pane.agent = Some(AgentSnapshot {
                            harness,
                            status: signal.status,
                            observed_status: signal.status,
                            integration_mode: IntegrationMode::Native,
                            activity_score: signal.activity_score,
                            debounce_ticks: 0,
                        });
                        summary.applied += 1;
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
        FileNativeSignalSource, NativeOverlaySummary, NativeSignalSource,
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
