use crate::services::control_api::{AgentsResponse, CONTROL_API_SCHEMA_VERSION};
use crate::sources::{SourceDescriptor, SourceDiagnostic, SourceError, SourceId, SourceResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub const SOURCE_SNAPSHOT_SCHEMA_VERSION: u16 = 1;
pub const SNAPSHOT_FRESH_MS: u128 = 2_000;
pub const SNAPSHOT_WARM_MS: u128 = 15_000;
pub const SNAPSHOT_STALE_MS: u128 = 5 * 60_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSnapshotEnvelope {
    pub schema_version: u16,
    pub source_id: String,
    pub captured_at_unix_ms: u128,
    pub expires_at_unix_ms: u128,
    pub agents_response_schema_version: u16,
    pub response: AgentsResponse,
    pub health: SourceSnapshotHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceSnapshotHealth {
    pub status: String,
    pub message: Option<String>,
}

impl SourceSnapshotEnvelope {
    pub fn new(source_id: &SourceId, response: AgentsResponse, now_ms: u128) -> Self {
        Self {
            schema_version: SOURCE_SNAPSHOT_SCHEMA_VERSION,
            source_id: source_id.as_str().to_string(),
            captured_at_unix_ms: now_ms,
            expires_at_unix_ms: now_ms + SNAPSHOT_STALE_MS,
            agents_response_schema_version: CONTROL_API_SCHEMA_VERSION,
            response,
            health: SourceSnapshotHealth {
                status: "ok".to_string(),
                message: None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotFreshness {
    Fresh,
    Warm,
    Stale,
}

#[derive(Debug, Clone)]
pub struct LoadedSourceSnapshot {
    pub envelope: SourceSnapshotEnvelope,
    pub freshness: SnapshotFreshness,
}

#[derive(Debug, Clone)]
pub struct SourceSnapshotStore {
    root: PathBuf,
}

impl SourceSnapshotStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn default_file_for(&self, source_id: &SourceId) -> PathBuf {
        self.root
            .join("sources")
            .join(format!("{}.agents.json", source_id.as_str()))
    }

    pub fn publish_snapshot(
        &self,
        source_id: &SourceId,
        envelope: &SourceSnapshotEnvelope,
    ) -> io::Result<PathBuf> {
        self.publish_snapshot_to(&self.default_file_for(source_id), envelope)
    }

    pub fn publish_snapshot_to(
        &self,
        path: &Path,
        envelope: &SourceSnapshotEnvelope,
    ) -> io::Result<PathBuf> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp_path = path.with_extension(format!(
            "tmp.{}.{}",
            std::process::id(),
            crate::sources::unix_ms_now()
        ));
        let payload = serde_json::to_vec_pretty(envelope).map_err(io::Error::other)?;
        {
            let mut file = fs::File::create(&tmp_path)?;
            file.write_all(&payload)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        fs::rename(&tmp_path, path)?;
        Ok(path.to_path_buf())
    }

    pub fn load_snapshot_for_source(
        &self,
        source_id: &SourceId,
        descriptor: &SourceDescriptor,
        path: &Path,
        now_ms: u128,
    ) -> SourceResult<LoadedSourceSnapshot> {
        let payload = fs::read_to_string(path).map_err(|error| {
            SourceError::new(
                "source.snapshot.unavailable",
                format!(
                    "snapshot source {} could not read {}: {error}",
                    source_id,
                    path.display()
                ),
                true,
            )
        })?;
        let envelope: SourceSnapshotEnvelope = serde_json::from_str(&payload).map_err(|error| {
            SourceError::new(
                "source.snapshot.invalid-json",
                format!(
                    "snapshot source {} returned invalid snapshot JSON: {error}",
                    source_id
                ),
                false,
            )
        })?;
        validate_envelope(source_id, &envelope)?;
        if now_ms > envelope.expires_at_unix_ms {
            return Err(SourceError::new(
                "source.snapshot.expired",
                format!(
                    "snapshot source {} expired {}ms ago",
                    source_id,
                    now_ms.saturating_sub(envelope.expires_at_unix_ms)
                ),
                true,
            ));
        }
        let age_ms = now_ms.saturating_sub(envelope.captured_at_unix_ms);
        let freshness = if age_ms <= SNAPSHOT_FRESH_MS {
            SnapshotFreshness::Fresh
        } else if age_ms <= SNAPSHOT_WARM_MS {
            SnapshotFreshness::Warm
        } else {
            SnapshotFreshness::Stale
        };
        let _ = descriptor;
        Ok(LoadedSourceSnapshot {
            envelope,
            freshness,
        })
    }

    pub fn stale_diagnostic(
        descriptor: &SourceDescriptor,
        freshness: SnapshotFreshness,
        captured_at_unix_ms: u128,
        now_ms: u128,
    ) -> Option<SourceDiagnostic> {
        if freshness != SnapshotFreshness::Stale {
            return None;
        }
        Some(SourceDiagnostic::warning(
            descriptor,
            "source.snapshot.stale",
            format!(
                "{} snapshot is stale: age={}ms",
                descriptor.label,
                now_ms.saturating_sub(captured_at_unix_ms)
            ),
            true,
            None,
        ))
    }
}

fn validate_envelope(source_id: &SourceId, envelope: &SourceSnapshotEnvelope) -> SourceResult<()> {
    if envelope.schema_version != SOURCE_SNAPSHOT_SCHEMA_VERSION {
        return Err(SourceError::new(
            "source.snapshot.schema-unsupported",
            format!(
                "snapshot source {} returned unsupported snapshot schema version {}",
                source_id, envelope.schema_version
            ),
            false,
        ));
    }
    if envelope.source_id != source_id.as_str() {
        return Err(SourceError::new(
            "source.snapshot.source-id-mismatch",
            format!(
                "snapshot source {} contained source id {}",
                source_id, envelope.source_id
            ),
            false,
        ));
    }
    if envelope.agents_response_schema_version != CONTROL_API_SCHEMA_VERSION
        || envelope.response.schema_version != CONTROL_API_SCHEMA_VERSION
    {
        return Err(SourceError::new(
            "source.snapshot.agents-schema-unsupported",
            format!(
                "snapshot source {} returned unsupported agents schema version {}",
                source_id, envelope.response.schema_version
            ),
            false,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::control_api::{AgentEntry, ControlInventorySummary};
    use crate::sources::{SourceConfig, SourceDescriptor, SourceDisplayConfig};

    fn response() -> AgentsResponse {
        AgentsResponse {
            schema_version: CONTROL_API_SCHEMA_VERSION,
            generated_at_unix_ms: 1,
            inventory: ControlInventorySummary {
                total_sessions: 1,
                total_windows: 1,
                total_panes: 1,
                visible_sessions: 1,
                visible_windows: 0,
                visible_panes: 1,
            },
            entries: vec![AgentEntry::test_entry("%42")],
            diagnostics: Vec::new(),
            sources: Vec::new(),
            source_diagnostics: Vec::new(),
            partial_failure_count: 0,
        }
    }

    fn descriptor() -> SourceDescriptor {
        let id = SourceId::new("mac");
        let config = SourceConfig::Snapshot {
            label: "Mac".to_string(),
            path: PathBuf::from("/tmp/mac.json"),
            enabled: true,
            display: SourceDisplayConfig::default(),
        };
        SourceDescriptor::new(&id, &config)
    }

    #[test]
    fn snapshot_store_round_trips_snapshot_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceSnapshotStore::new(dir.path());
        let source_id = SourceId::new("mac");
        let envelope = SourceSnapshotEnvelope::new(&source_id, response(), 1_000);

        let path = store.publish_snapshot(&source_id, &envelope).unwrap();
        let loaded = store
            .load_snapshot_for_source(&source_id, &descriptor(), &path, 1_500)
            .unwrap();

        assert_eq!(loaded.envelope.source_id, "mac");
        assert_eq!(loaded.envelope.response.entries[0].pane_id, "%42");
        assert_eq!(loaded.freshness, SnapshotFreshness::Fresh);
    }

    #[test]
    fn snapshot_store_rejects_source_id_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceSnapshotStore::new(dir.path());
        let source_id = SourceId::new("mac");
        let mut envelope = SourceSnapshotEnvelope::new(&source_id, response(), 1_000);
        envelope.source_id = "coder".to_string();
        let path = store
            .publish_snapshot_to(&dir.path().join("bad.json"), &envelope)
            .unwrap();

        let error = store
            .load_snapshot_for_source(&source_id, &descriptor(), &path, 1_500)
            .expect_err("mismatch should fail");

        assert_eq!(error.code, "source.snapshot.source-id-mismatch");
    }

    #[test]
    fn snapshot_store_expires_old_snapshots() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceSnapshotStore::new(dir.path());
        let source_id = SourceId::new("mac");
        let mut envelope = SourceSnapshotEnvelope::new(&source_id, response(), 1_000);
        envelope.expires_at_unix_ms = 2_000;
        let path = store.publish_snapshot(&source_id, &envelope).unwrap();

        let error = store
            .load_snapshot_for_source(&source_id, &descriptor(), &path, 2_001)
            .expect_err("expired should fail");

        assert_eq!(error.code, "source.snapshot.expired");
    }

    #[test]
    fn snapshot_store_marks_stale_but_unexpired_snapshots() {
        let dir = tempfile::tempdir().unwrap();
        let store = SourceSnapshotStore::new(dir.path());
        let source_id = SourceId::new("mac");
        let envelope = SourceSnapshotEnvelope::new(&source_id, response(), 1_000);
        let path = store.publish_snapshot(&source_id, &envelope).unwrap();

        let loaded = store
            .load_snapshot_for_source(&source_id, &descriptor(), &path, 100_000)
            .unwrap();

        assert_eq!(loaded.freshness, SnapshotFreshness::Stale);
        assert!(SourceSnapshotStore::stale_diagnostic(
            &descriptor(),
            loaded.freshness,
            loaded.envelope.captured_at_unix_ms,
            100_000,
        )
        .is_some());
    }
}
