use vigil_collect::{Collector, Health};

use super::bench::{Bench, said};

#[test]
fn a_host_where_no_path_is_named_says_there_is_nothing_to_check_and_reads_nothing() {
    let bench = Bench::new("named-nothing");
    let collector = bench.collector(&[], 1024);

    let health = collector.available();

    assert!(matches!(health, Health::Unavailable(_)), "{health:?}");
    assert!(
        said(&health).contains("no path is named"),
        "{}",
        said(&health)
    );
    assert!(
        collector.collect().is_err(),
        "an empty reading would be stored as a host whose files are all as they were"
    );
}

#[test]
fn a_watched_file_is_read_with_its_content_its_mode_and_its_owner() {
    let bench = Bench::new("one-file");
    let path = bench.write("sshd_config", "PermitRootLogin no\n");
    let collector = bench.collector(std::slice::from_ref(&path), 1024 * 1024);

    let reading = collector.collect().expect("reads");
    let row = &reading.items[&format!("file|{path}")];

    assert_eq!(collector.available(), Health::Ok);
    assert_eq!(row["present"], true);
    assert_eq!(row["readable"], true);
    assert_eq!(row["size"], 19);
    assert_eq!(row["mode"], "0644");
    assert_eq!(
        row["sha256"].as_str().map(str::len),
        Some(64),
        "the digest is what says the content changed, and nothing else in the row does"
    );
}

#[test]
fn a_file_rewritten_with_the_same_content_reads_the_same_however_often_it_is_written() {
    let bench = Bench::new("rewritten");
    let path = bench.write("hosts", "127.0.0.1 localhost\n");
    let collector = bench.collector(&[path], 1024 * 1024);

    let first = collector.collect().expect("reads");
    bench.write("hosts", "127.0.0.1 localhost\n");
    let later = collector.collect().expect("reads");

    assert_eq!(
        first.items, later.items,
        "a configuration manager writes the same file on every run, and a reading that moves \
         with the timestamp would be a finding on every run of it"
    );
}

#[test]
fn a_path_that_is_not_there_is_a_row_saying_so_and_the_reading_is_still_taken() {
    let bench = Bench::new("absent");
    let there = bench.write("hosts", "127.0.0.1 localhost\n");
    let missing = bench.directory.join("not-here").display().to_string();
    let collector = bench.collector(&[there.clone(), missing.clone()], 1024 * 1024);

    let reading = collector.collect().expect("reads");

    assert_eq!(reading.items[&format!("file|{missing}")]["present"], false);
    assert_eq!(reading.items[&format!("file|{there}")]["present"], true);
    assert_eq!(collector.available(), Health::Ok);
}

#[test]
fn a_file_over_the_ceiling_is_marked_rather_than_hashed_and_the_health_says_why() {
    let bench = Bench::new("ceiling");
    let path = bench.write("big", &"x".repeat(4096));
    let collector = bench.collector(std::slice::from_ref(&path), 1024);

    let reading = collector.collect().expect("reads");
    let row = &reading.items[&format!("file|{path}")];
    let health = collector.available();

    assert_eq!(row["over_the_ceiling"], true);
    assert_eq!(row["sha256"], serde_json::Value::Null);
    assert_eq!(row["size"], 4096);
    assert!(matches!(health, Health::Degraded(_)), "{health:?}");
    assert!(said(&health).contains("4096"), "{}", said(&health));
}

#[test]
fn a_directory_of_the_path_is_read_with_the_mode_that_says_who_may_write_a_program_into_it() {
    let bench = Bench::new("path");
    let path = bench.write("hosts", "127.0.0.1 localhost\n");
    let collector = bench.collector(&[path], 1024 * 1024);

    let reading = collector.collect().expect("reads");
    let row = &reading.items[&format!("directory|{}", bench.directory.display())];

    assert_eq!(row["present"], true);
    assert!(row["mode"].as_str().is_some());
}

#[test]
fn a_mode_that_changed_is_the_only_thing_that_moved_in_the_row() {
    let bench = Bench::new("chmod");
    let path = bench.write("sshd_config", "PermitRootLogin no\n");
    let collector = bench.collector(std::slice::from_ref(&path), 1024 * 1024);

    let before = collector.collect().expect("reads");
    bench.chmod("sshd_config", 0o4755);
    let after = collector.collect().expect("reads");

    let key = format!("file|{path}");
    assert_eq!(before.items[&key]["mode"], "0644");
    assert_eq!(after.items[&key]["mode"], "4755");
    assert_eq!(before.items[&key]["sha256"], after.items[&key]["sha256"]);
}
