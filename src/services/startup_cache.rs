use crate::app::Inventory;
use crate::config::RuntimeConfig;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const STARTUP_CACHE_VERSION: u32 = 1;

#[derive(Debug)]
pub enum StartupCacheError {
    Io(io::Error),
    Parse(serde_json::Error),
    Serialize(serde_json::Error),
}

impl fmt::Display for StartupCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::Parse(error) => write!(f, "{error}"),
            Self::Serialize(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StartupCacheError {}

impl From<io::Error> for StartupCacheError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedStartupCache {
    pub inventory: Inventory,
    pub generated_at_ms: u64,
    pub age_ms: u64,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupCacheInventorySummary {
    pub sessions: usize,
    pub windows: usize,
    pub panes: usize,
}

impl StartupCacheInventorySummary {
    fn from_inventory(inventory: &Inventory) -> Self {
        Self {
            sessions: inventory.session_count(),
            windows: inventory.window_count(),
            panes: inventory.pane_count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartupCacheLoadResult {
    Loaded(LoadedStartupCache),
    Missing {
        path: PathBuf,
    },
    Stale {
        path: PathBuf,
        age_ms: u64,
        inventory: StartupCacheInventorySummary,
    },
    Empty {
        path: PathBuf,
    },
    Invalid {
        path: PathBuf,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupCacheWriteReceipt {
    pub path: PathBuf,
    pub generated_at_ms: u64,
    pub bytes_written: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct StartupCacheFile {
    version: u32,
    server_identity: String,
    generated_at_ms: u64,
    inventory: Inventory,
}

pub fn startup_cache_file_path(cache_dir: &Path, tmux_socket: Option<&Path>) -> PathBuf {
    cache_dir.join(format!(
        "startup-{}.json",
        tmux_server_cache_key(tmux_socket)
    ))
}

pub fn load_startup_cache(
    runtime: &RuntimeConfig,
    max_age_ms: u64,
) -> Result<StartupCacheLoadResult, StartupCacheError> {
    let path = startup_cache_file_path(&runtime.startup_cache_dir, runtime.tmux_socket.as_deref());
    if !path.exists() {
        return Ok(StartupCacheLoadResult::Missing { path });
    }

    let contents = fs::read_to_string(&path)?;
    let cache_file: StartupCacheFile = match serde_json::from_str(&contents) {
        Ok(cache_file) => cache_file,
        Err(error) => {
            return Ok(StartupCacheLoadResult::Invalid {
                path,
                reason: error.to_string(),
            })
        }
    };
    if cache_file.version != STARTUP_CACHE_VERSION {
        return Ok(StartupCacheLoadResult::Invalid {
            path,
            reason: format!("unsupported version {}", cache_file.version),
        });
    }

    let expected_server_identity = tmux_server_identity(runtime.tmux_socket.as_deref());
    if cache_file.server_identity != expected_server_identity {
        return Ok(StartupCacheLoadResult::Invalid {
            path,
            reason: "server identity mismatch".to_string(),
        });
    }

    if cache_file.inventory.sessions.is_empty() {
        return Ok(StartupCacheLoadResult::Empty { path });
    }

    let now_ms = current_time_ms();
    let age_ms = now_ms.saturating_sub(cache_file.generated_at_ms);
    if age_ms > max_age_ms {
        return Ok(StartupCacheLoadResult::Stale {
            path,
            age_ms,
            inventory: StartupCacheInventorySummary::from_inventory(&cache_file.inventory),
        });
    }

    Ok(StartupCacheLoadResult::Loaded(LoadedStartupCache {
        inventory: cache_file.inventory,
        generated_at_ms: cache_file.generated_at_ms,
        age_ms,
        path,
    }))
}

pub fn write_startup_cache(
    runtime: &RuntimeConfig,
    inventory: &Inventory,
) -> Result<StartupCacheWriteReceipt, StartupCacheError> {
    let path = startup_cache_file_path(&runtime.startup_cache_dir, runtime.tmux_socket.as_deref());
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("startup cache path has no parent"))?;
    fs::create_dir_all(parent)?;

    let generated_at_ms = current_time_ms();
    let cache_file = StartupCacheFile {
        version: STARTUP_CACHE_VERSION,
        server_identity: tmux_server_identity(runtime.tmux_socket.as_deref()),
        generated_at_ms,
        inventory: inventory.clone(),
    };
    let serialized = serde_json::to_vec(&cache_file).map_err(StartupCacheError::Serialize)?;
    let temp_path = parent.join(format!(
        ".startup-cache-{}-{}.tmp",
        std::process::id(),
        generated_at_ms
    ));
    fs::write(&temp_path, &serialized)?;
    fs::rename(&temp_path, &path)?;

    Ok(StartupCacheWriteReceipt {
        path,
        generated_at_ms,
        bytes_written: serialized.len(),
    })
}

pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the unix epoch")
        .as_millis() as u64
}

fn tmux_server_identity(tmux_socket: Option<&Path>) -> String {
    tmux_socket
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "default".to_string())
}

fn tmux_server_cache_key(tmux_socket: Option<&Path>) -> String {
    let mut hasher = DefaultHasher::new();
    tmux_server_identity(tmux_socket).hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::{
        load_startup_cache, startup_cache_file_path, write_startup_cache, StartupCacheLoadResult,
    };
    use crate::app::{
        inventory, HarnessKind, PaneBuilder, SessionBuilder, SortMode, WindowBuilder,
    };
    use crate::config::{
        IntegrationPreference, LogVerbosity, NotificationBackendName, RuntimeConfig,
    };
    use crate::ui::theme::ThemeName;
    use std::path::Path;
    use tempfile::tempdir;

    fn runtime_config(root: &Path) -> RuntimeConfig {
        RuntimeConfig {
            config_file: root.join("config.toml"),
            log_dir: root.join("logs"),
            startup_cache_dir: root.join("cache"),
            ui_preferences_file: root.join("ui-state.json"),
            tmux_socket: Some(root.join("tmux.sock")),
            claude_native_dir: None,
            codex_native_dir: None,
            pi_native_dir: None,
            log_verbosity: LogVerbosity::Info,
            poll_interval_ms: 1500,
            capture_lines: 40,
            startup_cache_max_age_ms: crate::config::DEFAULT_STARTUP_CACHE_MAX_AGE_MS,
            popup: true,
            pull_request_monitoring_enabled: true,
            pull_request_poll_interval_ms: 30_000,
            notifications_enabled: true,
            notification_cooldown_ticks: 3,
            notification_backends: vec![
                NotificationBackendName::NotifySend,
                NotificationBackendName::OsaScript,
            ],
            notification_profile: crate::app::NotificationProfile::All,
            claude_integration_preference: IntegrationPreference::Auto,
            codex_integration_preference: IntegrationPreference::Auto,
            pi_integration_preference: IntegrationPreference::Auto,
            log_retention: 5,
            theme: ThemeName::Catppuccin,
            default_sort: SortMode::Stable,
            ui_theme_configured: false,
            ui_default_sort_configured: false,
        }
    }

    fn sample_inventory() -> crate::app::Inventory {
        inventory([SessionBuilder::new("alpha").window(
            WindowBuilder::new("alpha:agents").pane(
                PaneBuilder::agent("alpha:claude", HarnessKind::ClaudeCode)
                    .working_dir("/tmp/alpha")
                    .preview("Claude Code ready"),
            ),
        )])
    }

    #[test]
    fn startup_cache_round_trip_loads_written_inventory() {
        let root = tempdir().expect("temp dir should exist");
        let runtime = runtime_config(root.path());
        let inventory = sample_inventory();

        let receipt = write_startup_cache(&runtime, &inventory).expect("cache write should work");
        let load = load_startup_cache(&runtime, 60_000).expect("cache load should work");

        match load {
            StartupCacheLoadResult::Loaded(loaded) => {
                assert_eq!(loaded.inventory, inventory);
                assert_eq!(loaded.path, receipt.path);
                assert!(loaded.age_ms < 60_000);
            }
            other => panic!("expected loaded cache, got {other:?}"),
        }
    }

    #[test]
    fn stale_startup_cache_keeps_inventory_counts_for_diagnostics() {
        let root = tempdir().expect("temp dir should exist");
        let runtime = runtime_config(root.path());
        let inventory = sample_inventory();
        write_startup_cache(&runtime, &inventory).expect("cache write should work");
        std::thread::sleep(std::time::Duration::from_millis(2));

        let load = load_startup_cache(&runtime, 0).expect("cache lookup should work");

        match load {
            StartupCacheLoadResult::Stale {
                inventory,
                age_ms: _,
                path: _,
            } => {
                assert_eq!(inventory.sessions, 1);
                assert_eq!(inventory.windows, 1);
                assert_eq!(inventory.panes, 1);
            }
            other => panic!("expected stale cache, got {other:?}"),
        }
    }

    #[test]
    fn startup_cache_returns_missing_when_absent() {
        let root = tempdir().expect("temp dir should exist");
        let runtime = runtime_config(root.path());

        let load = load_startup_cache(&runtime, 60_000).expect("cache lookup should work");

        match load {
            StartupCacheLoadResult::Missing { path } => {
                assert_eq!(
                    path,
                    startup_cache_file_path(
                        &runtime.startup_cache_dir,
                        runtime.tmux_socket.as_deref()
                    )
                );
            }
            other => panic!("expected missing cache, got {other:?}"),
        }
    }
}
