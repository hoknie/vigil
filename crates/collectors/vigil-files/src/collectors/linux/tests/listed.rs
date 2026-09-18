use vigil_collect::{Collector, Health};

use super::bench::{Bench, said};

#[test]
fn a_watch_list_that_is_not_there_is_said_plainly_and_is_never_a_reading_of_nothing() {
    let bench = Bench::new("list-absent");
    let collector = bench.listed("watch_fs.yaml", 100);

    let health = collector.available();

    assert!(matches!(health, Health::Unavailable(_)), "{health:?}");
    assert!(said(&health).contains("is not there"), "{}", said(&health));
    assert!(said(&health).contains("watch_fs.yaml"), "{}", said(&health));
    assert!(
        collector.collect().is_err(),
        "an empty reading would be stored as a host whose files are all as they were"
    );
}

#[test]
fn a_file_named_in_a_watch_list_reads_exactly_as_it_read_from_the_configuration() {
    let bench = Bench::new("list-same-row");
    let hosts = bench.write("hosts", "127.0.0.1 localhost\n");
    bench.write("watch_fs.yaml", &format!("files:\n  - {hosts}\n"));

    let from_the_list = bench.listed("watch_fs.yaml", 100).collect().expect("reads");
    let from_the_configuration = bench
        .collector(std::slice::from_ref(&hosts), 1024 * 1024)
        .collect()
        .expect("reads");

    let key = format!("file|{hosts}");
    assert_eq!(
        from_the_list.items[&key], from_the_configuration.items[&key],
        "a host that moves its list out of vigil.yaml keeps every row it held, or the move is \
         a change on every watched file"
    );
}

#[test]
fn a_path_added_to_the_watch_list_is_read_at_the_next_reading_with_no_restart() {
    let bench = Bench::new("list-edited");
    let hosts = bench.write("hosts", "127.0.0.1 localhost\n");
    let motd = bench.write("motd", "welcome\n");
    bench.write("watch_fs.yaml", &format!("files:\n  - {hosts}\n"));
    let collector = bench.listed("watch_fs.yaml", 100);

    let before = collector.collect().expect("reads");
    bench.write(
        "watch_fs.yaml",
        &format!("files:\n  - {hosts}\n  - path: {motd}\n    max_file_size: 4kb\n"),
    );
    let after = collector.collect().expect("reads");

    assert!(!before.items.contains_key(&format!("file|{motd}")));
    assert_eq!(after.items[&format!("file|{motd}")]["ceiling_bytes"], 4096);
}

#[test]
fn a_watch_list_broken_by_an_edit_keeps_what_it_named_and_the_health_names_the_file() {
    let bench = Bench::new("list-broken");
    let hosts = bench.write("hosts", "127.0.0.1 localhost\n");
    let list = bench.write("watch_fs.yaml", &format!("files:\n  - {hosts}\n"));
    let collector = bench.listed("watch_fs.yaml", 100);
    collector.collect().expect("reads");

    bench.write("watch_fs.yaml", "files:\n  - [not, a, path]\n");
    let reading = collector
        .collect()
        .expect("the last good list is still read");
    let health = collector.available();

    assert!(reading.items.contains_key(&format!("file|{hosts}")));
    assert!(matches!(health, Health::Degraded(_)), "{health:?}");
    assert!(said(&health).contains(&list), "{}", said(&health));
}
