use super::*;
use crate::parsers::SessionSource;
use sessions::read_sessions;

#[test]
fn reads_this_host_and_names_itself_in_the_snapshot() {
    let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

    let snapshot = collector
        .collect()
        .expect("/etc/passwd is readable on Linux");

    assert_eq!(snapshot.source, "users");
    assert_eq!(snapshot.taken_at, "2026-09-09T12:00:00.000Z");
    assert!(
        snapshot.items.keys().any(|key| key.starts_with("account|")),
        "every host has accounts"
    );
    for (key, item) in &snapshot.items {
        assert!(key.contains('|'), "key shape: {key}");
        if key.starts_with("account|") {
            assert!(
                item.get("shadow_readable").is_some(),
                "{key} lost the flag that separates 'could not read' from 'not locked'"
            );
        }
    }
}

#[test]
fn nothing_that_could_be_cracked_reaches_the_snapshot_of_this_host() {
    let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());
    let snapshot = collector.collect().expect("readable");

    let printed = serde_json::to_string(&snapshot).expect("serialises");
    for marker in ["$1$", "$5$", "$6$", "$y$", "$2b$"] {
        assert!(
            !printed.contains(marker),
            "a password hash marker reached the snapshot: {marker}"
        );
    }
}

#[test]
fn every_source_of_logins_this_build_knows_leaves_a_row_whatever_this_host_holds() {
    let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());
    let snapshot = collector.collect().expect("readable");

    for name in ["utmp", "logind"] {
        let row = &snapshot.items[&format!("session-source|{name}")];
        assert!(row["present"].is_boolean(), "{name}: {row}");
        assert!(row["read"].is_boolean(), "{name}: {row}");
        assert!(
            row["present"] == serde_json::Value::Bool(true) || row["reason"].is_string(),
            "{name} is not on this host and does not say why: {row}"
        );
    }
}

#[test]
fn a_session_that_is_on_this_host_is_one_row_naming_the_sources_that_saw_it() {
    let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());
    let snapshot = collector.collect().expect("readable");

    for (key, item) in &snapshot.items {
        if !key.starts_with("session|") {
            continue;
        }
        let seen = item["seen_by"].as_array().expect("a list of sources");
        assert!(!seen.is_empty(), "{key} came from nowhere: {item}");
        for source in seen {
            assert!(
                ["utmp", "logind"].contains(&source.as_str().unwrap_or_default()),
                "{key} names a source this build does not read: {source}"
            );
        }
    }
}

#[test]
fn the_collector_says_which_sources_it_has_rather_than_that_it_reads_none() {
    let collector = UsersCollector::new(|| "2026-09-09T12:00:00.000Z".to_string());

    if let Health::Degraded(reason) = collector.available() {
        assert!(
            !reason.contains("in a form this build reads"),
            "the old wording said nothing a person could act on: {reason}"
        );
    }
}

#[test]
fn a_host_without_a_login_source_is_degraded_and_one_with_a_quiet_source_is_not() {
    let none = [
        SessionSource::absent("utmp", "/run/utmp", "not there"),
        SessionSource::absent("logind", "/run/systemd/sessions", "not there"),
    ];
    let quiet = [
        SessionSource::absent("utmp", "/run/utmp", "not there"),
        SessionSource::read("logind", "/run/systemd/sessions", 0),
    ];

    let said = health::session_notes(&none);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("/run/utmp"), "{said:?}");
    assert!(said[0].contains("/run/systemd/sessions"), "{said:?}");
    assert!(
        said[0].chars().count() < 120,
        "the state of the host has to fit on the screen beside the other collectors: {said:?}"
    );

    assert!(
        health::session_notes(&quiet).is_empty(),
        "a source that was read and held nothing is not a fault"
    );
}

#[test]
fn a_source_that_is_there_and_refused_is_named_even_when_the_other_one_answers() {
    let mixed = [
        SessionSource::refused("utmp", "/run/utmp", "/run/utmp could not be read: denied"),
        SessionSource::read("logind", "/run/systemd/sessions", 2),
    ];

    let said = health::session_notes(&mixed);
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said[0].contains("/run/utmp"), "{said:?}");
}

#[test]
fn reading_the_sessions_of_this_host_names_a_source_for_every_one_of_them() {
    let seen = read_sessions(&[]);

    assert_eq!(seen.sources.len(), 2);
    for session in &seen.sessions {
        assert!(!session.sources.is_empty(), "{session:?}");
        assert!(session.key().starts_with("session|"), "{session:?}");
    }
    let counted: usize = seen.sources.iter().map(|source| source.sessions).sum();
    assert!(
        counted >= seen.sessions.len(),
        "merging may join rows, never invent them"
    );
}

#[test]
fn the_sudoers_of_a_host_that_keeps_it_under_usr_etc_names_both_directories_it_includes() {
    let tumbleweed = "\
## Read drop-in files
@includedir /usr/etc/sudoers.d
@includedir /etc/sudoers.d
root ALL=(ALL:ALL) ALL
";

    let included = sudoers::included_by(tumbleweed, std::path::Path::new("/usr/etc"));

    assert_eq!(
        sudoers::directories_to_read(&included),
        vec![
            std::path::PathBuf::from("/usr/etc/sudoers.d"),
            std::path::PathBuf::from("/etc/sudoers.d"),
        ],
        "openSUSE Tumbleweed ships /usr/etc/sudoers and no /etc/sudoers, and a grant dropped \
         into /usr/etc/sudoers.d is one sudo honours"
    );
}

#[test]
fn the_old_hash_spelling_of_an_include_is_an_include_and_not_a_comment() {
    let debian = "\
Defaults env_reset
#includedir /etc/sudoers.d
#include /etc/sudoers.local
# includedir /not/a/directive
";

    let included = sudoers::included_by(debian, std::path::Path::new("/etc"));

    assert_eq!(
        included.directories,
        vec![std::path::PathBuf::from("/etc/sudoers.d")]
    );
    assert_eq!(
        included.files,
        vec![std::path::PathBuf::from("/etc/sudoers.local")]
    );
}

#[test]
fn an_include_named_relative_to_the_sudoers_file_is_read_beside_it() {
    let included = sudoers::included_by(
        "@include \"sudoers.local\"\n@includedir sudoers.d\n",
        std::path::Path::new("/usr/etc"),
    );

    assert_eq!(
        included.files,
        vec![std::path::PathBuf::from("/usr/etc/sudoers.local")]
    );
    assert_eq!(
        included.directories,
        vec![std::path::PathBuf::from("/usr/etc/sudoers.d")]
    );
}

#[test]
fn etc_sudoers_d_is_read_even_when_the_sudoers_file_could_not_say_it_includes_it() {
    assert_eq!(
        sudoers::directories_to_read(&sudoers::Included::default()),
        vec![std::path::PathBuf::from(SUDOERS_DIRECTORY)],
        "an agent that cannot read /etc/sudoers still lists the drop-in directory every \
         distribution includes, rather than seeing no grant at all"
    );
}

#[test]
fn an_include_whose_path_depends_on_the_host_name_is_not_guessed_at() {
    let included = sudoers::included_by("@include /etc/sudoers.%h\n", std::path::Path::new("/etc"));

    assert!(included.files.is_empty());
}
