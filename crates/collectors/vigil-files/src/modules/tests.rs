use serde_json::json;
use vigil_module::{Module, Settings};

use super::Files;
use crate::types::{WATCHED_BY_DEFAULT, Watched, Watching};

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_a_watched_path_walks_to_the_row_keyed_by_that_path() {
    for key in ["file|/etc/ssh/sshd_config", "directory|/usr/local/bin"] {
        assert_eq!(
            Files.row_of(key),
            Some(key.to_string()),
            "this collector keys its rows the way its rules key their findings, and cutting \
             the first segment would look for a row that was never written"
        );
    }
    assert!(!Files.raised("resource|disk|/var"));
    assert_eq!(Files.row_of("resource|disk|/var"), None);
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Files.name(), "files");
    assert_eq!(Files.every_seconds(), 300);
    assert_eq!(Files.unit(), None);
    assert!(!Files.rules(&Settings::plain(at_noon)).is_empty());
    assert!(Files.section().is_some());
}

#[test]
fn a_host_whose_file_names_no_path_watches_the_list_this_product_ships() {
    let watching = Watching::default();

    let paths: Vec<&str> = watching.paths.iter().map(Watched::path).collect();
    assert!(paths.contains(&"/etc/ssh/sshd_config"));
    assert_eq!(watching.ceiling_bytes, 1024 * 1024);
    for path in &paths {
        assert!(
            !path.ends_with('/'),
            "{path} is a directory, and a directory in this list is a walk that hashes \
             everything under it: the shortest path to an agent the operator turns off"
        );
    }
    for (path, hashed) in watching.hashed() {
        assert_eq!(
            hashed,
            1024 * 1024,
            "{path} names no ceiling of its own, so it is hashed to the one the block names"
        );
    }
}

#[test]
fn a_host_that_names_its_own_paths_watches_those_and_not_the_shipped_ones() {
    let named = Settings::of(
        at_noon,
        "files",
        serde_json::json!({ "paths": ["/etc/sudoers"] }),
    );
    let watching: Watching = named.read().expect("the sample parses");

    assert_eq!(
        watching.paths,
        vec![Watched::Named("/etc/sudoers".to_string())]
    );
    assert_eq!(
        watching.ceiling_bytes,
        1024 * 1024,
        "a file that names the paths leaves the ceiling on the one this module ships"
    );
}

#[test]
fn a_list_that_would_watch_nothing_is_refused_at_the_door_and_not_at_the_first_reading() {
    for (said, complained_about) in [
        (json!({"ceiling_bytes": 0}), "ceiling_bytes"),
        (json!({"paths": ["etc/hosts"]}), "etc/hosts"),
    ] {
        let refusal = Files
            .check(&Settings::of(at_noon, "files", said.clone()))
            .expect_err("a list this collector cannot read is a list nobody meant to write");

        assert!(refusal.contains(complained_about), "{said}: {refusal}");
    }
}

#[test]
fn a_key_this_module_does_not_know_is_refused_rather_than_quietly_left_out() {
    let refusal = Files
        .check(&Settings::of(
            at_noon,
            "files",
            json!({"path": ["/etc/hosts"]}),
        ))
        .expect_err("a misspelled key leaves the shipped list watched, and nothing says so");

    assert!(refusal.contains("path"), "{refusal}");
}

#[test]
fn a_host_that_names_no_path_at_all_is_a_file_this_product_reads() {
    let none = Settings::of(at_noon, "files", json!({"paths": []}));

    assert!(
        Files.check(&none).is_ok(),
        "naming no path is a decision an operator is allowed to make; what it must not be is \
         a reading that says every file is as it was, and the collector answers that with \
         Unavailable rather than with an empty snapshot"
    );
}

#[test]
fn none_of_the_paths_this_product_ships_is_one_another_collector_already_reports_on() {
    for already in [
        "/etc/passwd",
        "/etc/shadow",
        "/etc/group",
        "/etc/sudoers",
        "/etc/crontab",
        "/etc/ld.so.preload",
    ] {
        assert!(
            !WATCHED_BY_DEFAULT.contains(&already),
            "{already} is read by another collector of this build, which reports what changed \
             in it by name; hashing it here as well would put two findings on one edit"
        );
    }
}

#[test]
fn a_path_that_names_a_ceiling_of_its_own_is_hashed_to_that_one_and_the_rest_to_the_block() {
    let named = Settings::of(
        at_noon,
        "files",
        json!({
            "paths": [
                "/etc/hosts",
                { "path": "/etc/ssl/certs/ca-certificates.crt", "ceiling_bytes": 8_388_608u64 },
            ],
        }),
    );
    let watching: Watching = named.read().expect("the sample parses");

    assert_eq!(
        watching.hashed(),
        vec![
            ("/etc/hosts".to_string(), 1024 * 1024),
            ("/etc/ssl/certs/ca-certificates.crt".to_string(), 8_388_608),
        ],
        "a bundle past the ceiling would be watched by its mode alone, and naming the ceiling \
         beside the path is how an operator says otherwise for that one file"
    );
    assert!(Files.check(&named).is_ok());
}

#[test]
fn a_path_named_twice_is_refused_rather_than_read_and_hashed_twice_every_pass() {
    let twice = Settings::of(
        at_noon,
        "files",
        json!({ "paths": ["/etc/hosts", { "path": "/etc/hosts", "ceiling_bytes": 4096 }] }),
    );

    let refusal = Files
        .check(&twice)
        .expect_err("two entries for one path read the same file twice on every pass");

    assert!(refusal.contains("named twice"), "{refusal}");
}

#[test]
fn a_ceiling_no_pass_of_this_agent_could_finish_is_refused_at_the_door() {
    let huge = Settings::of(
        at_noon,
        "files",
        json!({ "paths": [{ "path": "/var/log/journal.bin", "ceiling_bytes": 1_073_741_824u64 }] }),
    );

    let refusal = Files.check(&huge).expect_err("must not be accepted");

    assert!(refusal.contains("paths #1"), "{refusal}");
    assert!(refusal.contains("over the"), "{refusal}");
}

#[test]
fn the_paths_this_module_watches_are_taken_from_the_file_again_while_the_daemon_runs() {
    assert!(
        Files.follows_the_file(),
        "a path added from the console is not watched until a restart otherwise, and the \
         operator who added it believes it already is"
    );
}
