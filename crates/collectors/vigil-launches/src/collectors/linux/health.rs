use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::Path;

use super::LaunchesCollector;
use super::advice::nothing_carries_our_tag;
use super::chunk::tail;
use crate::parsers::{AUDIT_KEY, any_launch_carries_our_tag, any_launch_was_read};
use crate::spool::{CEILING_BYTES, dropped_note};
use vigil_collect::Health;

const HEALTH_TAIL: u64 = 256 * 1024;

impl LaunchesCollector {
    pub(super) fn health(&self) -> Health {
        let spool = self.spool_bytes();
        if spool > 0 {
            if let Some(note) = self.dropped() {
                return Health::Degraded(format!(
                    "the audit plugin {note}. Launches from that window are not visible; later ones are. Usual cause: this daemon was stopped while the plugin kept writing. The spool at {} holds {CEILING_BYTES} bytes and drops the oldest first",
                    self.spool_path.display()
                ));
            }
            if !self.the_rule_is_standing(&self.spool_path, spool) {
                return Health::Degraded(nothing_carries_our_tag(&self.spool_path));
            }
            return Health::Ok;
        }

        let length = match fs::metadata(&self.log_path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Health::Unavailable(format!(
                    "auditd is not running: program launches are not visible (no {}, and nothing at {})",
                    self.log_path.display(),
                    self.spool_path.display()
                ));
            }
            Err(error) => {
                return Health::Unavailable(format!(
                    "{} cannot be read ({error}): program launches are not visible; the file is 0640 root:adm",
                    self.log_path.display()
                ));
            }
        };

        if !self.the_rule_is_standing(&self.log_path, length) {
            return Health::Degraded(nothing_carries_our_tag(&self.log_path));
        }
        if self.plugin_is_registered() {
            return match self.spool_path.exists() {
                false => Health::Degraded(format!(
                    "auditd is recording launches and the plugin registered in {} has never run: there is no {} at all. auditd starts its plugins at its own start, so one installed since then is not running yet; run `systemctl restart auditd`. Until it runs, launches are read from {} instead: one reading late, and a rotation between two readings takes what it held",
                    self.plugin_config.display(),
                    self.spool_path.display(),
                    self.log_path.display()
                )),
                true => Health::Degraded(format!(
                    "the plugin registered in {} has delivered nothing: {} is empty while {} carries records with the {AUDIT_KEY} tag. Either nobody has run a command since the plugin started, or auditd has not been restarted since this file was installed and an older plugin is running; `auditctl -l | grep {AUDIT_KEY}` and the plugin's line in auditd's log tell the two apart. Launches are read from the log meanwhile",
                    self.plugin_config.display(),
                    self.spool_path.display(),
                    self.log_path.display()
                )),
            };
        }
        Health::Ok
    }

    pub(super) fn dropped(&self) -> Option<String> {
        let mut file = File::open(&self.spool_path).ok()?;
        let mut head = [0u8; 512];
        let read = file.read(&mut head).ok()?;
        let line = head[..read]
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        dropped_note(line)
    }

    fn plugin_is_registered(&self) -> bool {
        let Ok(text) = fs::read_to_string(&self.plugin_config) else {
            return false;
        };
        text.lines().any(|line| {
            let line = line.trim();
            let Some((name, value)) = line.split_once('=') else {
                return false;
            };
            name.trim().eq_ignore_ascii_case("active") && value.trim().eq_ignore_ascii_case("yes")
        })
    }

    fn the_rule_is_standing(&self, path: &Path, length: u64) -> bool {
        self.what_this_agent_has_read() || self.carries_our_records(path, length)
    }

    fn what_this_agent_has_read(&self) -> bool {
        let seen = self
            .seen
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        seen.rule_loaded || any_launch_was_read(&seen.items)
    }

    fn carries_our_records(&self, path: &Path, length: u64) -> bool {
        match tail(path, length, HEALTH_TAIL) {
            Ok(tail) => any_launch_carries_our_tag(&tail),
            Err(_) => false,
        }
    }
}
