use super::{
    DisplayError, DisplayIdentity, DisplayProviderKind, SourceDisplayRegistration,
    SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION,
};
use crate::config::AppPaths;
use crate::sources::SourceId;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DisplayRegistryEnvelope {
    schema_version: u16,
    #[serde(default)]
    registrations: BTreeMap<String, SourceDisplayRegistration>,
}

impl Default for DisplayRegistryEnvelope {
    fn default() -> Self {
        Self {
            schema_version: SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION,
            registrations: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceDisplayRegistry {
    path: PathBuf,
}

impl SourceDisplayRegistry {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn for_paths(paths: &AppPaths) -> Self {
        Self::for_log_dir(&paths.log_dir)
    }

    pub fn for_log_dir(log_dir: &Path) -> Self {
        let state_dir = log_dir.parent().unwrap_or(log_dir);
        Self::new(state_dir.join("source-displays.json"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get(
        &self,
        source_id: &SourceId,
    ) -> Result<Option<SourceDisplayRegistration>, DisplayError> {
        Ok(self.load()?.registrations.get(source_id.as_str()).cloned())
    }

    pub fn list(&self) -> Result<Vec<SourceDisplayRegistration>, DisplayError> {
        Ok(self.load()?.registrations.into_values().collect())
    }

    pub fn register(
        &self,
        source_id: &SourceId,
        provider: DisplayProviderKind,
        identity: DisplayIdentity,
        now_ms: u128,
    ) -> Result<SourceDisplayRegistration, DisplayError> {
        identity.validate()?;
        let _lock = RegistryLock::acquire(&self.lock_path())?;
        let mut envelope = self.load()?;
        let registration = SourceDisplayRegistration {
            source_id: source_id.as_str().to_string(),
            provider,
            identity,
            captured_at_unix_ms: now_ms,
            updated_at_unix_ms: now_ms,
            ownership_handle: new_opaque_handle()?,
        };
        envelope
            .registrations
            .insert(source_id.as_str().to_string(), registration.clone());
        self.atomic_write(&envelope)?;
        Ok(registration)
    }

    pub fn unregister(
        &self,
        source_id: &SourceId,
        ownership_handle: &str,
    ) -> Result<bool, DisplayError> {
        let _lock = RegistryLock::acquire(&self.lock_path())?;
        let mut envelope = self.load()?;
        let Some(current) = envelope.registrations.get(source_id.as_str()) else {
            return Ok(false);
        };
        if current.ownership_handle != ownership_handle {
            return Err(DisplayError::new(
                "source.display.ownership-mismatch",
                format!(
                    "display registration for source {} has a different ownership handle; list registrations and retry with the current handle",
                    source_id
                ),
                false,
            ));
        }
        envelope.registrations.remove(source_id.as_str());
        self.atomic_write(&envelope)?;
        Ok(true)
    }

    fn load(&self) -> Result<DisplayRegistryEnvelope, DisplayError> {
        let payload = match fs::read_to_string(&self.path) {
            Ok(payload) => payload,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(DisplayRegistryEnvelope::default())
            }
            Err(error) => return Err(error.into()),
        };
        let envelope: DisplayRegistryEnvelope =
            serde_json::from_str(&payload).map_err(|error| {
                DisplayError::new(
                    "source.display.registry-invalid",
                    format!(
                        "display registry {} is not valid JSON: {error}",
                        self.path.display()
                    ),
                    false,
                )
            })?;
        if envelope.schema_version != SOURCE_DISPLAY_REGISTRY_SCHEMA_VERSION {
            return Err(DisplayError::new(
                "source.display.registry-schema-unsupported",
                format!(
                    "display registry {} uses unsupported schema version {}",
                    self.path.display(),
                    envelope.schema_version
                ),
                false,
            ));
        }
        for (key, registration) in &envelope.registrations {
            if key != &registration.source_id {
                return Err(DisplayError::new(
                    "source.display.registry-source-id-mismatch",
                    format!(
                        "display registry entry {key} contains source id {}",
                        registration.source_id
                    ),
                    false,
                ));
            }
            registration.identity.validate()?;
        }
        Ok(envelope)
    }

    fn atomic_write(&self, envelope: &DisplayRegistryEnvelope) -> Result<(), DisplayError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp_path = self.path.with_extension(format!(
            "json.{}.{}.tmp",
            std::process::id(),
            new_opaque_handle()?
        ));
        let payload = serde_json::to_vec_pretty(envelope).map_err(|error| {
            DisplayError::new(
                "source.display.registry-invalid",
                format!("failed to encode display registry: {error}"),
                false,
            )
        })?;
        {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(&payload)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }
        fs::rename(&temp_path, &self.path)?;
        Ok(())
    }

    fn lock_path(&self) -> PathBuf {
        self.path.with_extension("lock")
    }
}

struct RegistryLock {
    file: fs::File,
}

impl RegistryLock {
    fn acquire(path: &Path) -> Result<Self, DisplayError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        file.lock_exclusive().map_err(DisplayError::from)?;
        Ok(Self { file })
    }
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

pub(super) fn new_opaque_handle() -> Result<String, DisplayError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        DisplayError::new(
            "source.display.registry-entropy",
            format!("failed to generate a display registration ownership handle: {error}"),
            true,
        )
    })?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
