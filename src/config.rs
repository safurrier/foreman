use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const APP_DIR_NAME: &str = "foreman";
const CONFIG_FILE_NAME: &str = "config.toml";
pub const DEFAULT_LOG_RETENTION: usize = 20;

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    MissingHomeDirectory(&'static str),
    ParseToml(toml::de::Error),
    SerializeToml(toml::ser::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::MissingHomeDirectory(kind) => {
                write!(f, "unable to resolve a user-scoped {kind} path")
            }
            Self::ParseToml(error) => write!(f, "{error}"),
            Self::SerializeToml(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::ParseToml(error)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(error: toml::ser::Error) -> Self {
        Self::SerializeToml(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathEnvironment {
    pub foreman_config_home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub appdata: Option<PathBuf>,
    pub foreman_log_dir: Option<PathBuf>,
    pub xdg_state_home: Option<PathBuf>,
    pub localappdata: Option<PathBuf>,
}

impl PathEnvironment {
    pub fn capture() -> Self {
        Self {
            foreman_config_home: std::env::var_os("FOREMAN_CONFIG_HOME").map(PathBuf::from),
            xdg_config_home: std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
            home: std::env::var_os("HOME").map(PathBuf::from),
            appdata: std::env::var_os("APPDATA").map(PathBuf::from),
            foreman_log_dir: std::env::var_os("FOREMAN_LOG_DIR").map(PathBuf::from),
            xdg_state_home: std::env::var_os("XDG_STATE_HOME").map(PathBuf::from),
            localappdata: std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_file: PathBuf,
    pub log_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub monitoring: MonitoringConfig,
    pub notifications: NotificationConfig,
    pub logging: LoggingConfig,
    pub pull_requests: PullRequestConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub poll_interval_ms: u64,
    pub capture_lines: usize,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1_000,
            capture_lines: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enabled: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub retain_run_logs: usize,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            retain_run_logs: DEFAULT_LOG_RETENTION,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestConfig {
    pub enabled: bool,
    pub poll_interval_ms: u64,
}

impl Default for PullRequestConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub config_file: PathBuf,
    pub log_dir: PathBuf,
    pub tmux_socket: Option<PathBuf>,
    pub claude_native_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub capture_lines: usize,
    pub popup: bool,
    pub notifications_enabled: bool,
    pub log_retention: usize,
}

impl RuntimeConfig {
    pub fn from_sources(paths: AppPaths, file_config: AppConfig, cli: &crate::cli::Cli) -> Self {
        Self {
            config_file: paths.config_file,
            log_dir: paths.log_dir,
            tmux_socket: cli.tmux_socket.clone(),
            claude_native_dir: cli.claude_native_dir.clone(),
            poll_interval_ms: cli
                .poll_interval_ms
                .unwrap_or(file_config.monitoring.poll_interval_ms),
            capture_lines: cli
                .capture_lines
                .unwrap_or(file_config.monitoring.capture_lines),
            popup: cli.popup,
            notifications_enabled: if cli.no_notify {
                false
            } else {
                file_config.notifications.enabled
            },
            log_retention: file_config.logging.retain_run_logs,
        }
    }
}

pub fn resolve_paths(
    config_file_override: Option<&Path>,
    log_dir_override: Option<&Path>,
) -> Result<AppPaths, ConfigError> {
    resolve_paths_with_env(
        config_file_override,
        log_dir_override,
        &PathEnvironment::capture(),
    )
}

pub fn resolve_paths_with_env(
    config_file_override: Option<&Path>,
    log_dir_override: Option<&Path>,
    env: &PathEnvironment,
) -> Result<AppPaths, ConfigError> {
    let config_file = match config_file_override {
        Some(path) => path.to_path_buf(),
        None => resolve_config_file(env)?,
    };

    let log_dir = match log_dir_override {
        Some(path) => path.to_path_buf(),
        None => resolve_log_dir(env)?,
    };

    Ok(AppPaths {
        config_file,
        log_dir,
    })
}

pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}

pub fn write_default_config(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(default_config_toml().as_bytes())?;
    Ok(())
}

pub fn default_config_toml() -> String {
    toml::to_string_pretty(&AppConfig::default()).expect("default config should serialize")
}

fn resolve_config_file(env: &PathEnvironment) -> Result<PathBuf, ConfigError> {
    let base_dir = if let Some(path) = &env.foreman_config_home {
        path.clone()
    } else if let Some(path) = &env.xdg_config_home {
        path.clone()
    } else if cfg!(windows) {
        env.appdata
            .clone()
            .ok_or(ConfigError::MissingHomeDirectory("config"))?
    } else {
        env.home
            .clone()
            .map(|home| home.join(".config"))
            .ok_or(ConfigError::MissingHomeDirectory("config"))?
    };

    Ok(base_dir.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
}

fn resolve_log_dir(env: &PathEnvironment) -> Result<PathBuf, ConfigError> {
    if let Some(path) = &env.foreman_log_dir {
        return Ok(path.clone());
    }

    let base_dir = if let Some(path) = &env.xdg_state_home {
        path.clone()
    } else if cfg!(windows) {
        env.localappdata
            .clone()
            .or_else(|| env.appdata.clone())
            .ok_or(ConfigError::MissingHomeDirectory("log"))?
    } else {
        env.home
            .clone()
            .map(|home| home.join(".local").join("state"))
            .ok_or(ConfigError::MissingHomeDirectory("log"))?
    };

    Ok(base_dir.join(APP_DIR_NAME).join("logs"))
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_paths_with_env, write_default_config, AppConfig, ConfigError, LoggingConfig,
        PathEnvironment, PullRequestConfig, RuntimeConfig,
    };
    use crate::cli::Cli;
    use clap::Parser;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn resolve_paths_prefers_explicit_overrides() {
        let env = PathEnvironment {
            foreman_config_home: Some(Path::new("/ignored/config").to_path_buf()),
            xdg_config_home: None,
            home: Some(Path::new("/ignored/home").to_path_buf()),
            appdata: None,
            foreman_log_dir: Some(Path::new("/ignored/logs").to_path_buf()),
            xdg_state_home: None,
            localappdata: None,
        };

        let paths = resolve_paths_with_env(
            Some(Path::new("/tmp/custom-config.toml")),
            Some(Path::new("/tmp/logs")),
            &env,
        )
        .expect("paths should resolve");

        assert_eq!(paths.config_file, Path::new("/tmp/custom-config.toml"));
        assert_eq!(paths.log_dir, Path::new("/tmp/logs"));
    }

    #[test]
    fn resolve_paths_uses_foreman_specific_environment_over_generic_home() {
        let env = PathEnvironment {
            foreman_config_home: Some(Path::new("/tmp/config-home").to_path_buf()),
            xdg_config_home: Some(Path::new("/tmp/xdg-config").to_path_buf()),
            home: Some(Path::new("/tmp/home").to_path_buf()),
            appdata: None,
            foreman_log_dir: Some(Path::new("/tmp/custom-logs").to_path_buf()),
            xdg_state_home: Some(Path::new("/tmp/xdg-state").to_path_buf()),
            localappdata: None,
        };

        let paths = resolve_paths_with_env(None, None, &env).expect("paths should resolve");

        assert_eq!(
            paths.config_file,
            Path::new("/tmp/config-home/foreman/config.toml")
        );
        assert_eq!(paths.log_dir, Path::new("/tmp/custom-logs"));
    }

    #[test]
    fn write_default_config_creates_parseable_toml() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_path = temp_dir.path().join("config.toml");

        write_default_config(&config_path).expect("config should be created");

        let parsed: AppConfig = toml::from_str(
            &std::fs::read_to_string(&config_path).expect("config should be readable"),
        )
        .expect("config should parse");

        assert_eq!(parsed.logging, LoggingConfig::default());
        assert_eq!(parsed.pull_requests, PullRequestConfig::default());
    }

    #[test]
    fn write_default_config_fails_if_file_already_exists() {
        let temp_dir = tempdir().expect("temp dir should exist");
        let config_path = temp_dir.path().join("config.toml");
        write_default_config(&config_path).expect("first write should succeed");

        let error = write_default_config(&config_path).expect_err("second write should fail");

        assert!(matches!(error, ConfigError::Io(_)));
    }

    #[test]
    fn runtime_config_applies_cli_overrides() {
        let cli = Cli::parse_from([
            "foreman",
            "--poll-interval-ms",
            "250",
            "--capture-lines",
            "400",
            "--popup",
            "--no-notify",
        ]);

        let runtime = RuntimeConfig::from_sources(
            super::AppPaths {
                config_file: Path::new("/tmp/config.toml").to_path_buf(),
                log_dir: Path::new("/tmp/logs").to_path_buf(),
            },
            AppConfig::default(),
            &cli,
        );

        assert_eq!(runtime.poll_interval_ms, 250);
        assert_eq!(runtime.capture_lines, 400);
        assert_eq!(runtime.tmux_socket, None);
        assert_eq!(runtime.claude_native_dir, None);
        assert!(runtime.popup);
        assert!(!runtime.notifications_enabled);
    }
}
