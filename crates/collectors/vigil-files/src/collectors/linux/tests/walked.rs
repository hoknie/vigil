use std::os::unix::fs::symlink;

use vigil_collect::{Collector, Health};
use vigil_rules::{RuleContext, diff};

use super::bench::{Bench, said};
use crate::rules::file_rules;

fn tree(bench: &Bench) -> String {
    bench.write("etc/pam.d/sshd", "auth required pam_unix.so\n");
    bench.write("etc/pam.d/su", "auth sufficient pam_rootok.so\n");
    bench.write("etc/pam.d/deeper/login", "auth required pam_env.so\n");
    bench.at("etc/pam.d")
}

#[test]
fn a_directory_in_the_watch_list_is_walked_whole_and_each_path_says_it_was_found_by_it() {
    let bench = Bench::new("walk-whole");
    let root = tree(&bench);
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}/\n"));

    let reading = bench.listed("watch_fs.yaml", 100).collect().expect("reads");

    let login = &reading.items[&format!("file|{root}/deeper/login")];
    assert_eq!(login["found_by"], root.as_str());
    assert_eq!(login["complete"], true);
    assert_eq!(login["sha256"].as_str().map(str::len), Some(64));
    assert_eq!(reading.items[&format!("file|{root}")]["type"], "directory");
    assert_eq!(
        reading.items[&format!("file|{root}/deeper")]["type"],
        "directory"
    );
    assert_eq!(
        reading.items[&format!("walk|{root}")]["matched"],
        4,
        "sshd, su, deeper and deeper/login"
    );
}

#[test]
fn a_link_inside_a_walk_is_a_row_saying_where_it_points_and_is_never_followed() {
    let bench = Bench::new("walk-link");
    let root = tree(&bench);
    bench.write("elsewhere/secret", "not under the watched directory\n");
    symlink(bench.at("elsewhere"), format!("{root}/outside")).expect("links");
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}\n"));

    let reading = bench.listed("watch_fs.yaml", 100).collect().expect("reads");

    let link = &reading.items[&format!("file|{root}/outside")];
    assert_eq!(link["type"], "symlink");
    assert_eq!(link["target"], bench.at("elsewhere").as_str());
    assert!(
        !reading
            .items
            .contains_key(&format!("file|{root}/outside/secret")),
        "a link followed is a walk that leaves the place it was asked to watch"
    );
}

#[test]
fn a_mask_matches_inside_one_name_and_a_mask_matching_nothing_is_shown_as_such() {
    let bench = Bench::new("walk-mask");
    bench.write("etc/ssh/a.conf", "a\n");
    bench.write("etc/ssh/b.conf", "b\n");
    bench.write("etc/ssh/sshd_config", "c\n");
    let ssh = bench.at("etc/ssh");
    bench.write(
        "watch_fs.yaml",
        &format!("files:\n  - {ssh}/*.conf\n  - {ssh}/*.nothing\n"),
    );

    let reading = bench.listed("watch_fs.yaml", 100).collect().expect("reads");

    assert!(reading.items.contains_key(&format!("file|{ssh}/a.conf")));
    assert!(reading.items.contains_key(&format!("file|{ssh}/b.conf")));
    assert!(
        !reading
            .items
            .contains_key(&format!("file|{ssh}/sshd_config"))
    );
    assert_eq!(
        reading.items[&format!("walk|{ssh}/*.nothing")]["matched"],
        0
    );
}

#[test]
fn a_walk_past_max_files_stops_says_where_and_names_the_entries_it_never_reached() {
    let bench = Bench::new("walk-limit");
    let root = tree(&bench);
    bench.write("later/one", "1\n");
    let later = bench.at("later");
    bench.write(
        "watch_fs.yaml",
        &format!("files:\n  - {root}\n  - {later}\n"),
    );
    let collector = bench.listed("watch_fs.yaml", 2);

    let reading = collector.collect().expect("reads what fits");
    let health = collector.available();

    assert_eq!(reading.items[&format!("walk|{root}")]["complete"], false);
    assert_eq!(reading.items[&format!("walk|{later}")]["complete"], false);
    assert!(matches!(health, Health::Degraded(_)), "{health:?}");
    assert!(
        said(&health).contains("max_files is 2"),
        "{}",
        said(&health)
    );
    assert!(said(&health).contains(&later), "{}", said(&health));
}

#[test]
fn a_walk_taken_twice_over_an_unchanged_tree_reads_the_same_rows() {
    let bench = Bench::new("walk-twice");
    let root = tree(&bench);
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}\n"));
    let collector = bench.listed("watch_fs.yaml", 3);

    let first = collector.collect().expect("reads");
    let again = collector.collect().expect("reads");

    assert_eq!(
        first.items, again.items,
        "a walk cut at its limit cuts at the same place every time, or the paths past it would \
         come and go between readings"
    );
}

#[test]
fn the_filesystem_of_the_kernel_is_never_walked_even_when_the_list_names_it() {
    let bench = Bench::new("walk-proc");
    bench.write(
        "watch_fs.yaml",
        "files:\n  - /proc/sys/kernel\n  - /proc/*/status\n",
    );

    let reading = bench.listed("watch_fs.yaml", 100).collect().expect("reads");

    let walked = &reading.items["walk|/proc/sys/kernel"];
    assert_eq!(walked["matched"], 0);
    assert!(
        walked["not_entered"][0]
            .as_str()
            .is_some_and(|said| said.contains("(proc)")),
        "{walked}"
    );
    assert_eq!(reading.items["walk|/proc/*/status"]["matched"], 0);
}

#[test]
fn a_file_dropped_into_a_watched_directory_between_two_readings_is_a_finding() {
    let bench = Bench::new("walk-finding");
    let root = tree(&bench);
    bench.write("watch_fs.yaml", &format!("files:\n  - {root}\n"));
    let collector = bench.listed("watch_fs.yaml", 100);

    let before = collector.collect().expect("reads");
    bench.write("etc/pam.d/backdoor", "auth sufficient pam_permit.so\n");
    let after = collector.collect().expect("reads");

    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-18T12:05:00.000Z".into(),
        mint_event_id: &mut mint,
    };
    let findings = file_rules().judge(&diff(&before, &after), &mut ctx);

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].finding_key, format!("file|{root}/backdoor"));
    assert_eq!(findings[0].kind.as_str(), "file.changed");
}
