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

const RULESET_CEILING_BYTES: u64 = 8 * 1024 * 1024;

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

    fn stale_after_seconds(&self) -> u64 {
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

#[cfg(test)]
mod tests {
    use std::fs::{File, FileTimes};
    use std::time::Duration;

    use super::*;

    const EMPTY_RULESET: &str =
        r#"{"nftables": [{"metainfo": {"version": "1.0.6", "json_schema_version": 1}}]}"#;

    const ONE_FILTER: &str = r#"{"nftables": [
        {"metainfo": {"version": "1.0.6", "json_schema_version": 1}},
        {"table": {"family": "inet", "name": "filter", "handle": 1}},
        {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
        {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 4, "expr": [{"accept": null}]}}
    ]}"#;

    struct Bench {
        directory: PathBuf,
    }

    impl Bench {
        fn new(named: &str) -> Bench {
            let directory = std::env::temp_dir().join(format!(
                "vigil-firewall-{named}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&directory);
            fs::create_dir_all(&directory).expect("a bench to read from");
            Bench { directory }
        }

        fn ruleset(&self) -> PathBuf {
            self.directory.join("ruleset.json")
        }

        fn legacy(&self) -> PathBuf {
            self.directory.join("ip_tables_names")
        }

        fn write(&self, ruleset: &str) {
            fs::write(self.ruleset(), ruleset).expect("writes");
        }

        fn aged(&self, seconds: u64) {
            let file = File::options()
                .write(true)
                .open(self.ruleset())
                .expect("opens");
            let when = SystemTime::now() - Duration::from_secs(seconds);
            file.set_times(FileTimes::new().set_modified(when))
                .expect("sets the time this reading was written");
        }

        fn collector(&self, legacy: bool) -> FirewallCollector {
            let sources: Vec<String> = match legacy {
                true => vec![self.legacy().display().to_string()],
                false => Vec::new(),
            };
            let sources: Vec<&str> = sources.iter().map(String::as_str).collect();

            FirewallCollector::with_paths(
                || "2026-09-11T12:00:00.000Z".to_string(),
                self.ruleset(),
                &sources,
            )
        }
    }

    impl Drop for Bench {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.directory);
        }
    }

    fn said(health: &Health) -> String {
        match health {
            Health::Ok => String::new(),
            Health::Degraded(detail) | Health::Unavailable(detail) => detail.clone(),
        }
    }

    #[test]
    fn a_reading_older_than_two_periods_says_how_old_it_is() {
        let bench = Bench::new("stale");
        bench.write(ONE_FILTER);
        let collector = bench.collector(false);

        assert_eq!(collector.available(), Health::Ok);

        let age = collector.stale_after_seconds() + 41;
        bench.aged(age);
        let health = collector.available();

        assert!(
            matches!(health, Health::Degraded(_)),
            "a reading nobody refreshed is still a reading, and it is not Ok: {health:?}"
        );
        assert!(
            said(&health).contains(&age.to_string()),
            "the age is the whole point of the complaint: {}",
            said(&health)
        );
        assert!(
            collector.collect().is_ok(),
            "the reading itself is still what the host last looked like"
        );
    }

    #[test]
    fn the_four_ways_this_file_can_fail_are_four_different_answers_and_none_of_them_is_no_rules() {
        let bench = Bench::new("four");
        let collector = bench.collector(false);

        let missing = collector.available();
        assert!(matches!(missing, Health::Unavailable(_)), "{missing:?}");
        assert!(matches!(collector.collect(), Err(CollectError::Absent(_))));

        bench.write("");
        let empty = collector.available();
        assert!(matches!(empty, Health::Degraded(_)), "{empty:?}");
        assert!(said(&empty).contains("empty"), "{}", said(&empty));
        assert!(collector.collect().is_err());

        bench.write("Error: could not process rule: Operation not permitted\n");
        let rubbish = collector.available();
        assert!(matches!(rubbish, Health::Degraded(_)), "{rubbish:?}");
        assert!(
            said(&rubbish).contains("Operation not permitted"),
            "the first line of the file is what tells an operator what happened: {}",
            said(&rubbish)
        );
        assert!(collector.collect().is_err());

        bench.write(ONE_FILTER);
        bench.aged(collector.stale_after_seconds() + 10);
        let stale = collector.available();
        assert!(matches!(stale, Health::Degraded(_)), "{stale:?}");

        for health in [&missing, &empty, &rubbish, &stale] {
            assert_ne!(*health, Health::Ok);
            assert!(
                said(health).contains("filter") || said(health).contains("written"),
                "every one of the four says what is not known: {}",
                said(health)
            );
        }
        assert_ne!(said(&missing), said(&empty));
        assert_ne!(said(&empty), said(&rubbish));
        assert_ne!(said(&rubbish), said(&stale));
    }

    #[test]
    fn rules_held_by_the_legacy_backend_are_degraded_and_not_absent() {
        let bench = Bench::new("legacy");
        bench.write(EMPTY_RULESET);
        fs::write(bench.legacy(), "filter\nnat\n").expect("writes");
        let collector = bench.collector(true);

        let health = collector.available();
        assert!(matches!(health, Health::Degraded(_)), "{health:?}");
        assert!(
            said(&health).contains("legacy backend"),
            "{}",
            said(&health)
        );

        let reading = collector.collect().expect("a reading is still taken");
        assert_eq!(reading.items["fw-summary|nftables"]["legacy_backend"], true);
        assert!(reading.items.contains_key("fw-backend|legacy"));
    }

    #[test]
    fn a_host_that_really_has_no_nftables_table_and_no_old_backend_is_healthy_and_says_zero() {
        let bench = Bench::new("bare");
        bench.write(EMPTY_RULESET);
        let collector = bench.collector(false);

        assert_eq!(collector.available(), Health::Ok);
        let reading = collector.collect().expect("reads");
        assert_eq!(reading.items["fw-summary|nftables"]["tables"], 0);
        assert_eq!(
            reading.items["fw-summary|nftables"]["legacy_backend"],
            false
        );
    }

    #[test]
    fn a_ruleset_bigger_than_this_collector_reads_is_refused_by_name_and_not_half_read() {
        let bench = Bench::new("ceiling");
        let mut padding = String::from(r#"{"nftables": [{"metainfo": {"comment": ""#);
        padding.push_str(&"x".repeat(RULESET_CEILING_BYTES as usize + 1));
        padding.push_str(r#""}}]}"#);
        bench.write(&padding);
        let collector = bench.collector(false);

        assert!(matches!(collector.collect(), Err(CollectError::Budget(_))));
        assert!(matches!(collector.available(), Health::Degraded(_)));
    }
}
