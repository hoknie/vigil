use std::fs;
use std::path::{Path, PathBuf};

use vigil_collect::{CollectError, Collector, Health, outside_the_sample};
use vigil_model::{Golden, Shape};

use super::FirewallCollector;
use super::interfaces::interfaces;
use crate::fixture::{pf_dump, unprivileged_dump};
use crate::types::{DUMP_FILE, FirewallDump};

const AT: &str = "2026-09-19T12:00:00.000Z";

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "vigil-firewall-macos-{}-{named}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(&at).expect("a directory of its own");
    at
}

fn written(at: &Path, dump: &FirewallDump) -> PathBuf {
    let path = at.join(DUMP_FILE);
    fs::write(&path, serde_json::to_vec_pretty(dump).expect("plain data")).expect("write");
    path
}

#[test]
fn a_dump_the_job_wrote_as_root_reads_as_pf_the_application_firewall_and_the_links_of_this_mac() {
    let at = workspace("root");
    let collector = FirewallCollector::with_path(|| AT.to_string(), written(&at, &pf_dump(true)));

    let read = collector.collect().expect("reads");

    assert_eq!(collector.available(), Health::Ok);
    assert_eq!(read.source, "firewall");
    assert!(read.items.contains_key("fw-summary|pf"));
    assert!(read.items.contains_key("fw-application|socketfilterfw"));
    assert!(
        read.items.contains_key("fw-interface|lo0"),
        "every Mac has a loopback"
    );
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_dump_taken_without_root_is_read_for_the_application_firewall_and_says_pf_is_unknown() {
    let at = workspace("unprivileged");
    let collector =
        FirewallCollector::with_path(|| AT.to_string(), written(&at, &unprivileged_dump()));

    let read = collector
        .collect()
        .expect("the Application Firewall still reads");

    assert!(
        !read.items.keys().any(|key| key.contains("|pf")),
        "pf that could not be read leaves no row, or the next reading that can read it reports \
         every table as new and the one after a refusal reports them all as removed"
    );
    match collector.available() {
        Health::Degraded(why) => {
            assert!(why.contains("pf could not be read"), "{why}");
            assert!(why.contains("Permission denied"), "{why}");
            assert!(
                why.contains("not the same as pf filtering nothing"),
                "{why}"
            );
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_mac_where_the_job_has_never_run_says_the_reading_is_unknown_and_names_the_job() {
    let at = workspace("never");
    let collector = FirewallCollector::with_path(|| AT.to_string(), at.join(DUMP_FILE));

    assert!(matches!(collector.collect(), Err(CollectError::Absent(_))));
    match collector.available() {
        Health::Unavailable(why) => {
            assert!(why.contains("launchd job vigil.firewall"), "{why}");
            assert!(why.contains("vigild collector firewall enable"), "{why}");
            assert!(!why.contains("systemctl"), "{why}");
        }
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_dump_older_than_twice_its_period_is_read_and_its_age_is_said() {
    let at = workspace("stale");
    let path = written(&at, &pf_dump(true));
    let long_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(4_000);
    fs::File::options()
        .write(true)
        .open(&path)
        .expect("the dump")
        .set_times(fs::FileTimes::new().set_modified(long_ago))
        .expect("an older mtime");

    let collector = FirewallCollector::with_path(|| AT.to_string(), path);

    assert!(collector.collect().is_ok());
    match collector.available() {
        Health::Degraded(why) => assert!(why.contains("was written"), "{why}"),
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_file_that_is_not_the_document_of_the_job_is_a_refusal_naming_it() {
    let at = workspace("rubbish");
    let path = at.join(DUMP_FILE);
    fs::write(&path, b"pfctl: command not found\n").expect("write");

    let collector = FirewallCollector::with_path(|| AT.to_string(), &path);

    assert!(collector.collect().is_err());
    match collector.available() {
        Health::Degraded(why) => assert!(why.contains(DUMP_FILE), "{why}"),
        other => panic!("{other:?}"),
    }
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn on_a_mac_the_collector_reads_the_file_the_launchd_job_writes() {
    let collector = FirewallCollector::new(|| AT.to_string(), false);

    assert_eq!(
        collector.dump_path,
        PathBuf::from("/usr/local/var/lib/vigil/firewall/firewall.json")
    );
}

#[test]
fn the_links_of_this_mac_are_read_with_the_address_each_one_answers_on() {
    let links = interfaces();
    let loopback = links
        .iter()
        .find(|link| link.name == "lo0")
        .expect("every Mac has lo0");

    assert!(
        loopback
            .addresses
            .iter()
            .any(|address| address == "127.0.0.1")
    );
    assert!(loopback.traffic.is_some());
    assert!(!loopback.the_way_out);
    assert_eq!(
        links.last().map(|link| link.name.as_str()),
        Some("lo0"),
        "the loopback is listed last, because it is the one nothing arrives on"
    );
}

#[test]
fn a_reading_of_this_mac_has_the_shape_of_the_sample_the_screen_was_built_on() {
    let at = workspace("shape");
    let read = FirewallCollector::with_path(|| AT.to_string(), written(&at, &pf_dump(true)))
        .counting(true)
        .collect()
        .expect("reads");
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("firewall-macos")
            .held()
            .expect("the published shape of a Mac"),
    )
    .expect("a shape");

    let drift = outside_the_sample(&Shape::of(&read), &sample, &[]);

    assert!(drift.is_empty(), "{drift:#?}");
    let _ = fs::remove_dir_all(&at);
}
