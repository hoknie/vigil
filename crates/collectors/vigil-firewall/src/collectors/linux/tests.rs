use std::fs::{self, File, FileTimes};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::collector::{FirewallCollector, RULESET_CEILING_BYTES};
use vigil_collect::{CollectError, Collector, Health};

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
