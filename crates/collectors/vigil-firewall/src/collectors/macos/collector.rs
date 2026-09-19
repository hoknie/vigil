use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};
use vigil_module::Module;

use super::interfaces::interfaces;
use crate::parsers::{
    MacosFirewallReading, Pf, Understood, macos_firewall_snapshot, parse_firewall_dump, understood,
};
use crate::types::{DUMP_FILE, FirewallDump, MACOS_DUMP_DIRECTORY};

pub(super) const DUMP_CEILING_BYTES: u64 = 8 * 1024 * 1024;

const STALE_AFTER_PERIODS: u64 = 2;

pub struct FirewallCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    pub(super) dump_path: PathBuf,
    counting: bool,
}

impl FirewallCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static, counting: bool) -> Self {
        FirewallCollector::with_path(now, Path::new(MACOS_DUMP_DIRECTORY).join(DUMP_FILE))
            .counting(counting)
    }

    pub fn with_path(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        dump_path: impl Into<PathBuf>,
    ) -> Self {
        FirewallCollector {
            now: Box::new(now),
            dump_path: dump_path.into(),
            counting: false,
        }
    }

    pub fn counting(self, counting: bool) -> Self {
        FirewallCollector { counting, ..self }
    }

    pub(super) fn stale_after_seconds(&self) -> u64 {
        u64::from(crate::Firewall.every_seconds()) * STALE_AFTER_PERIODS
    }

    pub(super) fn dump(&self) -> Result<(FirewallDump, Option<u64>), CollectError> {
        let shown = self.dump_path.display();

        let metadata = fs::metadata(&self.dump_path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => CollectError::Absent(shown.to_string()),
            ErrorKind::PermissionDenied => CollectError::Denied(shown.to_string()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;
        if metadata.len() > DUMP_CEILING_BYTES {
            return Err(CollectError::Budget(format!(
                "{shown}: {} bytes, over the {DUMP_CEILING_BYTES} this collector reads",
                metadata.len()
            )));
        }

        let bytes = fs::read(&self.dump_path).map_err(|error| match error.kind() {
            ErrorKind::PermissionDenied => CollectError::Denied(shown.to_string()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;
        let dump = parse_firewall_dump(&bytes)
            .map_err(|refusal| CollectError::Unreadable(format!("{shown}: {refusal}")))?;

        Ok((dump, self.age_seconds(&metadata)))
    }

    fn age_seconds(&self, metadata: &fs::Metadata) -> Option<u64> {
        let written_at = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(written_at).ok()?.as_secs();

        match age > self.stale_after_seconds() {
            true => Some(age),
            false => None,
        }
    }
}

impl Collector for FirewallCollector {
    fn name(&self) -> &'static str {
        "firewall"
    }

    fn available(&self) -> Health {
        self.health()
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let (dump, _) = self.dump()?;
        let Understood {
            pf,
            application_firewall,
        } = understood(&dump);

        if let (Err(pf), Err(application)) = (&pf, &application_firewall) {
            return Err(CollectError::Unreadable(format!(
                "{}: neither firewall of this Mac could be read: {pf}; {application}",
                self.dump_path.display()
            )));
        }

        Ok(macos_firewall_snapshot(
            &(self.now)(),
            &MacosFirewallReading {
                pf: pf.as_ref().ok().map(|read| Pf {
                    enabled: read.enabled,
                    main: &read.main,
                    anchors: &read.anchors,
                }),
                application_firewall: application_firewall.as_ref().ok(),
                interfaces: &interfaces(),
                counting: self.counting,
            },
        ))
    }
}
