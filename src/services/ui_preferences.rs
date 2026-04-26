use crate::app::{AppState, Filters, HarnessKind, SelectionTarget, SessionId, SortMode};
use crate::ui::theme::ThemeName;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedUiPreferences {
    pub sort_mode: Option<SortMode>,
    pub theme: Option<ThemeName>,
    pub filters: Option<PersistedFilters>,
    pub collapsed_sessions: Vec<SessionId>,
    pub selection: Option<SelectionTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedFilters {
    pub show_non_agent_sessions: bool,
    pub show_non_agent_panes: bool,
    pub harness: Option<HarnessKind>,
}

impl Default for PersistedFilters {
    fn default() -> Self {
        Self::from(Filters::default())
    }
}

impl From<Filters> for PersistedFilters {
    fn from(filters: Filters) -> Self {
        Self {
            show_non_agent_sessions: filters.show_non_agent_sessions,
            show_non_agent_panes: filters.show_non_agent_panes,
            harness: filters.harness,
        }
    }
}

impl From<PersistedFilters> for Filters {
    fn from(filters: PersistedFilters) -> Self {
        Self {
            show_non_agent_sessions: filters.show_non_agent_sessions,
            show_non_agent_panes: filters.show_non_agent_panes,
            harness: filters.harness,
        }
    }
}

impl PersistedUiPreferences {
    pub fn from_state(state: &AppState, theme: ThemeName) -> Self {
        Self {
            sort_mode: Some(state.sort_mode),
            theme: Some(theme),
            filters: Some(PersistedFilters::from(state.filters.clone())),
            collapsed_sessions: state.collapsed_sessions.iter().cloned().collect(),
            selection: state.selection.clone(),
        }
    }

    pub fn apply_to_state(&self, state: &mut AppState) {
        if let Some(sort_mode) = self.sort_mode {
            state.sort_mode = sort_mode;
        }
        if let Some(filters) = self.filters {
            state.filters = filters.into();
        }
        state.collapsed_sessions = self
            .collapsed_sessions
            .iter()
            .filter(|session_id| state.inventory.contains_session(session_id))
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(selection) = &self.selection {
            if state.inventory.contains_target(selection) {
                state.selection = Some(selection.clone());
            }
        }
        state.rebuild_visible_state();
        state.reconcile_selection();
    }
}

pub fn apply_preferences_to_runtime(
    runtime: &mut crate::config::RuntimeConfig,
    preferences: &PersistedUiPreferences,
) {
    if !runtime.ui_theme_configured {
        if let Some(theme) = preferences.theme {
            runtime.theme = theme;
        }
    }
    if !runtime.ui_default_sort_configured {
        if let Some(sort_mode) = preferences.sort_mode {
            runtime.default_sort = sort_mode;
        }
    }
}

pub fn load_ui_preferences(path: &Path) -> io::Result<PersistedUiPreferences> {
    if !path.exists() {
        return Ok(PersistedUiPreferences::default());
    }

    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save_ui_preferences(path: &Path, preferences: &PersistedUiPreferences) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_vec_pretty(preferences)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = temp_preferences_path(path);
    let bytes_written = contents.len();
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(bytes_written)
}

pub fn reset_ui_preferences(path: &Path) -> io::Result<bool> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn temp_preferences_path(path: &Path) -> PathBuf {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("ui-state.json");
    parent.join(format!(
        ".{file_name}-{}-{}.tmp",
        std::process::id(),
        current_time_nanos()
    ))
}

fn current_time_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the unix epoch")
        .as_nanos()
}

#[cfg(test)]
mod tests {
    use super::{load_ui_preferences, save_ui_preferences, PersistedUiPreferences};
    use crate::app::{HarnessKind, SelectionTarget, SortMode};
    use crate::ui::theme::ThemeName;
    use tempfile::tempdir;

    #[test]
    fn ui_preferences_round_trip() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join("ui-state.json");
        let preferences = PersistedUiPreferences {
            sort_mode: Some(SortMode::AttentionFirst),
            theme: Some(ThemeName::Dracula),
            filters: Some(super::PersistedFilters {
                show_non_agent_sessions: true,
                show_non_agent_panes: false,
                harness: Some(HarnessKind::CodexCli),
            }),
            collapsed_sessions: vec!["alpha".into()],
            selection: Some(SelectionTarget::Pane("%1".into())),
        };

        let bytes_written = save_ui_preferences(&path, &preferences).expect("save should work");
        assert!(bytes_written > 0);
        let loaded = load_ui_preferences(&path).expect("load should work");

        assert_eq!(loaded, preferences);
    }
    #[test]
    fn reset_ui_preferences_removes_existing_file_and_tolerates_missing_file() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let path = temp_dir.path().join("ui-state.json");
        save_ui_preferences(&path, &PersistedUiPreferences::default()).expect("save should work");

        assert!(super::reset_ui_preferences(&path).expect("reset should work"));
        assert!(!path.exists());
        assert!(!super::reset_ui_preferences(&path).expect("second reset should work"));
    }
}
