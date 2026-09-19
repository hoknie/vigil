use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use vigil_model::Rfc3339;

use super::LaunchesCollector;
use super::auditd::audit_log;
use super::seen::Seen;
use crate::spool::{PLUGIN_CONFIG_PATH, SPOOL_PATH};

pub const AUDIT_LOG: &str = "/var/log/audit/audit.log";

impl LaunchesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static, keep_arguments: bool) -> Self {
        Self::with_paths(
            now,
            keep_arguments,
            SPOOL_PATH,
            audit_log(),
            PLUGIN_CONFIG_PATH,
        )
    }

    pub fn with_paths(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        keep_arguments: bool,
        spool_path: impl Into<PathBuf>,
        log_path: impl Into<PathBuf>,
        plugin_config: impl Into<PathBuf>,
    ) -> Self {
        LaunchesCollector {
            now: Box::new(now),
            spool_path: spool_path.into(),
            log_path: log_path.into(),
            plugin_config: plugin_config.into(),
            keep_arguments,
            seen: Mutex::new(Seen::default()),
        }
    }

    pub(super) fn spool_bytes(&self) -> u64 {
        fs::metadata(&self.spool_path)
            .map(|at| at.len())
            .unwrap_or(0)
    }
}
