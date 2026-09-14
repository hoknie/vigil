use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::SystemTime;

use vigil_model::{Rfc3339, Snapshot};

use crate::parsers::{
    FirewallReading, IP_TABLES_NAMES, IP6_TABLES_NAMES, NftRuleset, firewall_snapshot,
    parse_ip_tables_names, parse_nft_ruleset,
};
use vigil_collect::{CollectError, Collector, Health};
use vigil_module::Module;

const NAME: &str = "firewall";

pub const RULESET_PATH: &str = "/var/lib/vigil/firewall/ruleset.json";

pub const WRITTEN_BY: &str = "vigil-firewall.timer";

pub(super) const RULESET_CEILING_BYTES: u64 = 8 * 1024 * 1024;

const STALE_AFTER_PERIODS: u64 = 2;

const HOW_IT_IS_WRITTEN: &str = "this reading is written by the vigil-firewall.timer unit, which runs /usr/sbin/nft --json list ruleset and nothing else; the agent never starts a program of its own";

const HELD_BY_THE_LEGACY_BACKEND: &str = "the ruleset here is held by the legacy backend: nftables lists no table and /proc/net/ip_tables_names lists";

pub struct FirewallCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    ruleset_path: PathBuf,
    legacy_sources: Vec<PathBuf>,
}

impl FirewallCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static) -> Self {
        FirewallCollector::with_paths(now, RULESET_PATH, &[IP_TABLES_NAMES, IP6_TABLES_NAMES])
    }

    pub fn with_paths(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        ruleset_path: impl Into<PathBuf>,
        legacy_sources: &[&str],
    ) -> Self {
        FirewallCollector {
            now: Box::new(now),
            ruleset_path: ruleset_path.into(),
            legacy_sources: legacy_sources.iter().map(PathBuf::from).collect(),
        }
    }

    pub(super) fn stale_after_seconds(&self) -> u64 {
        u64::from(crate::Firewall.every_seconds()) * STALE_AFTER_PERIODS
    }

    fn ruleset(&self) -> Result<(NftRuleset, Option<u64>), CollectError> {
        let shown = self.ruleset_path.display();

        let metadata = fs::metadata(&self.ruleset_path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => CollectError::Absent(shown.to_string()),
            ErrorKind::PermissionDenied => CollectError::Denied(shown.to_string()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;

        if metadata.len() > RULESET_CEILING_BYTES {
            return Err(CollectError::Budget(format!(
                "{shown}: {} bytes, over the {RULESET_CEILING_BYTES} this collector reads",
                metadata.len()
            )));
        }

        let bytes = fs::read(&self.ruleset_path)
            .map_err(|error| CollectError::Unreadable(format!("{shown}: {error}")))?;

        let ruleset = parse_nft_ruleset(&bytes)
            .map_err(|refusal| CollectError::Unreadable(format!("{shown}: {refusal}")))?;

        Ok((ruleset, self.age_seconds(&metadata)))
    }

    fn age_seconds(&self, metadata: &fs::Metadata) -> Option<u64> {
        let written_at = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(written_at).ok()?.as_secs();

        match age > self.stale_after_seconds() {
            true => Some(age),
            false => None,
        }
    }

    fn legacy_tables(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .legacy_sources
            .iter()
            .filter_map(|path| fs::read_to_string(path).ok())
            .flat_map(|text| parse_ip_tables_names(&text))
            .collect();

        names.sort_unstable();
        names.dedup();
        names
    }

    fn refused(&self, refusal: &CollectError) -> Health {
        let shown = self.ruleset_path.display();
        match refusal {
            CollectError::Absent(_) => Health::Unavailable(format!(
                "{shown} is not there, so what this host filters is unknown — which is not the same as nothing. {HOW_IT_IS_WRITTEN}. Either the timer has never run (systemctl enable --now {WRITTEN_BY}), it is masked, or /usr/sbin/nft is not installed here"
            )),
            CollectError::Denied(_) => Health::Unavailable(format!(
                "{shown} cannot be read by this agent, so what this host filters is unknown. The directory is 0700 root:root and so is the file"
            )),
            other => Health::Degraded(format!(
                "{other}. {HOW_IT_IS_WRITTEN}; its last run left this behind. `systemctl status {WRITTEN_BY}` and the journal of vigil-firewall.service say why"
            )),
        }
    }

    fn stale(&self, age: u64) -> Health {
        Health::Degraded(format!(
            "the ruleset in {} was written {age} seconds ago, more than the {} this collector allows: what it says about this host may have been true and no longer is. {HOW_IT_IS_WRITTEN}; a timer that is not firing is the usual cause",
            self.ruleset_path.display(),
            self.stale_after_seconds()
        ))
    }

    fn legacy(&self, names: &[String]) -> Health {
        Health::Degraded(format!(
            "{HELD_BY_THE_LEGACY_BACKEND} {}. This build reads the nftables ruleset and does not parse iptables-save, so the rules on this host are not visible here. That is a reading this agent cannot take, not a host without a firewall",
            names.join(", ")
        ))
    }
}

impl Collector for FirewallCollector {
    fn name(&self) -> &'static str {
        NAME
    }

    fn available(&self) -> Health {
        let (ruleset, age) = match self.ruleset() {
            Ok(read) => read,
            Err(refusal) => return self.refused(&refusal),
        };

        if let Some(age) = age {
            return self.stale(age);
        }

        let legacy = self.legacy_tables();
        match ruleset.tables.is_empty() && !legacy.is_empty() {
            true => self.legacy(&legacy),
            false => Health::Ok,
        }
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let (ruleset, _) = self.ruleset()?;
        let legacy = self.legacy_tables();

        Ok(firewall_snapshot(
            &(self.now)(),
            &FirewallReading {
                ruleset: &ruleset,
                legacy_tables: &legacy,
            },
        ))
    }
}
