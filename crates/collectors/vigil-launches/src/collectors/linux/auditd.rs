use std::fs;
use std::path::PathBuf;

use super::source::AUDIT_LOG;

pub(super) const AUDITD_CONF: &str = "/etc/audit/auditd.conf";

pub(super) fn audit_log() -> PathBuf {
    fs::read_to_string(AUDITD_CONF)
        .ok()
        .and_then(|text| log_file_of(&text))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(AUDIT_LOG))
}

pub(super) fn log_file_of(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .find(|(key, _)| key.trim() == "log_file")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| value.starts_with('/'))
}
