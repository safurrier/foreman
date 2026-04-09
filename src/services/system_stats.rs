use std::fmt;

use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SystemStatsSnapshot {
    pub cpu_pressure_percent: Option<u16>,
    pub memory_pressure_percent: Option<u16>,
}

impl SystemStatsSnapshot {
    pub fn label(&self) -> String {
        format!(
            "cpu={} mem={}",
            format_percent(self.cpu_pressure_percent),
            format_percent(self.memory_pressure_percent)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemStatsError {
    Unavailable(String),
}

impl fmt::Display for SystemStatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for SystemStatsError {}

pub trait SystemStatsBackend {
    fn snapshot(&mut self) -> Result<SystemStatsSnapshot, SystemStatsError>;
}

#[derive(Debug, Clone)]
pub struct SystemStatsService<B> {
    backend: B,
}

impl<B> SystemStatsService<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
}

impl<B: SystemStatsBackend> SystemStatsService<B> {
    pub fn snapshot(&mut self) -> Result<SystemStatsSnapshot, SystemStatsError> {
        self.backend.snapshot()
    }
}

#[derive(Debug, Default)]
pub struct SysinfoSystemStatsBackend {
    system: System,
}

impl SysinfoSystemStatsBackend {
    pub fn new() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl SystemStatsBackend for SysinfoSystemStatsBackend {
    fn snapshot(&mut self) -> Result<SystemStatsSnapshot, SystemStatsError> {
        self.system.refresh_memory();

        let total_memory = self.system.total_memory();
        let used_memory = self.system.used_memory();
        let memory_pressure_percent = if total_memory == 0 {
            None
        } else {
            Some(((used_memory.saturating_mul(100)) / total_memory) as u16)
        };

        let cpu_count = std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1);
        let load_average = System::load_average();
        let cpu_pressure = ((load_average.one / cpu_count as f64) * 100.0).round();
        let cpu_pressure_percent = if cpu_pressure.is_finite() && cpu_pressure >= 0.0 {
            Some(cpu_pressure.min(u16::MAX as f64) as u16)
        } else {
            None
        };

        Ok(SystemStatsSnapshot {
            cpu_pressure_percent,
            memory_pressure_percent,
        })
    }
}

fn format_percent(value: Option<u16>) -> String {
    value
        .map(|value| format!("{value}%"))
        .unwrap_or_else(|| "?".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        SysinfoSystemStatsBackend, SystemStatsBackend, SystemStatsError, SystemStatsService,
        SystemStatsSnapshot,
    };

    #[derive(Debug)]
    struct FakeBackend {
        snapshot: Result<SystemStatsSnapshot, SystemStatsError>,
    }

    impl SystemStatsBackend for FakeBackend {
        fn snapshot(&mut self) -> Result<SystemStatsSnapshot, SystemStatsError> {
            self.snapshot.clone()
        }
    }

    #[test]
    fn snapshot_label_uses_question_mark_for_missing_values() {
        let snapshot = SystemStatsSnapshot::default();

        assert_eq!(snapshot.label(), "cpu=? mem=?");
    }

    #[test]
    fn service_returns_backend_snapshot() {
        let mut service = SystemStatsService::new(FakeBackend {
            snapshot: Ok(SystemStatsSnapshot {
                cpu_pressure_percent: Some(42),
                memory_pressure_percent: Some(67),
            }),
        });

        let snapshot = service.snapshot().expect("snapshot should succeed");

        assert_eq!(
            snapshot,
            SystemStatsSnapshot {
                cpu_pressure_percent: Some(42),
                memory_pressure_percent: Some(67),
            }
        );
        assert_eq!(snapshot.label(), "cpu=42% mem=67%");
    }

    #[test]
    fn sysinfo_backend_returns_soft_snapshot() {
        let mut backend = SysinfoSystemStatsBackend::new();

        let snapshot = backend.snapshot().expect("sysinfo snapshot should succeed");

        assert!(snapshot.cpu_pressure_percent.is_some());
        assert!(snapshot.memory_pressure_percent.is_some());
    }
}
