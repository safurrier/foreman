use crate::app::PaneId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const LINKED_REPOSITORIES_FILE_NAME: &str = "linked-repositories.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkedRepositoryLink {
    pub repository: PathBuf,
    pub pane_working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LinkedRepositories {
    pub links: BTreeMap<PaneId, LinkedRepositoryLink>,
}

impl LinkedRepositories {
    pub fn target_for(&self, pane_id: &PaneId) -> Option<&PathBuf> {
        self.links.get(pane_id).map(|link| &link.repository)
    }

    pub fn insert(&mut self, pane_id: PaneId, repo: PathBuf, pane_working_dir: Option<PathBuf>) {
        self.links.insert(
            pane_id,
            LinkedRepositoryLink {
                repository: repo,
                pane_working_dir,
            },
        );
    }

    pub fn remove(&mut self, pane_id: &PaneId) -> bool {
        self.links.remove(pane_id).is_some()
    }
}

pub fn linked_repositories_file(config_file: &Path) -> PathBuf {
    config_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(LINKED_REPOSITORIES_FILE_NAME)
}

pub fn load_linked_repositories(config_file: &Path) -> io::Result<LinkedRepositories> {
    let path = linked_repositories_file(config_file);
    if !path.exists() {
        return Ok(LinkedRepositories::default());
    }
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save_linked_repositories(config_file: &Path, links: &LinkedRepositories) -> io::Result<()> {
    let path = linked_repositories_file(config_file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = serde_json::to_vec_pretty(links)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let temp_path = path.with_extension(format!("json.{}.tmp", std::process::id()));
    fs::write(&temp_path, contents)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

pub fn resolve_git_repository(path: &Path) -> io::Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("{} is not inside a git repository", path.display())
        } else {
            stderr
        };
        return Err(io::Error::new(io::ErrorKind::InvalidInput, message));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(stdout.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linked_repositories_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = dir.path().join("config.toml");
        let mut links = LinkedRepositories::default();
        links.insert(
            PaneId::new("%1"),
            PathBuf::from("/tmp/repo"),
            Some(PathBuf::from("/tmp/notes")),
        );

        save_linked_repositories(&config, &links).expect("save links");
        let loaded = load_linked_repositories(&config).expect("load links");

        assert_eq!(
            loaded.target_for(&PaneId::new("%1")),
            Some(&PathBuf::from("/tmp/repo"))
        );
        assert_eq!(
            loaded
                .links
                .get(&PaneId::new("%1"))
                .and_then(|link| link.pane_working_dir.as_ref()),
            Some(&PathBuf::from("/tmp/notes"))
        );
    }
}
