use vigil_config::silenced;

use super::{Options, RESTART, add, list, remove};

const SHIPPED: &str = "state_dir: /var/lib/vigil\nsuppressions: []\nreporters: []\n";

fn temporary(name: &str) -> String {
    let directory = std::env::temp_dir().join(format!(
        "vigil-silence-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&directory).expect("temp dir");
    let path = directory.join(name);
    std::fs::write(&path, SHIPPED).expect("writes");
    path.to_str().expect("utf-8").to_string()
}

fn silencing(path: &str, key: &str) -> Options {
    Options {
        keys: vec![key.to_string()],
        reason: "the staging api, expected here".into(),
        path: path.to_string(),
        ..Options::default()
    }
}

#[test]
fn an_entry_written_here_is_one_the_daemon_reads_back_and_the_rest_of_the_file_is_untouched() {
    let path = temporary("write.yaml");

    let said = add(&silencing(&path, "port.listen|tcp|0.0.0.0:4444")).expect("writes");

    let after = std::fs::read_to_string(&path).expect("readable");
    assert!(after.contains("state_dir: /var/lib/vigil"), "{after}");
    assert!(after.contains("reporters: []"), "{after}");
    let held = silenced(&after).expect("the daemon would read it");
    assert_eq!(held.len(), 1);
    assert!(held[0].covers(
        "port.listen|tcp|0.0.0.0:4444",
        "port.listen.new",
        "2026-09-13T10:00:00.000Z"
    ));
    assert!(
        said.said.iter().any(|line| line.contains(RESTART)),
        "an entry that is not read until a restart, with nobody told to restart, is an \
         operator watching the same finding come back: {said:#?}"
    );
}

#[test]
fn one_object_named_twice_on_the_command_line_is_asked_for_once() {
    let path = temporary("twice.yaml");

    let done = add(&Options {
        keys: vec!["user|group|docker".into(), "user|group|docker".into()],
        ..silencing(&path, "unused")
    })
    .expect("writes");

    assert_eq!(done.entries, 1, "{:#?}", done.said);
    assert_eq!(
        std::fs::read_to_string(&path)
            .expect("readable")
            .matches("user|group|docker")
            .count(),
        1
    );
}

#[test]
fn what_was_written_is_taken_out_again_and_the_file_is_the_one_it_started_as() {
    let path = temporary("round.yaml");
    add(&silencing(&path, "user|group|docker")).expect("writes");

    remove(&Options {
        keys: vec!["user|group|docker".into()],
        path: path.clone(),
        ..Options::default()
    })
    .expect("writes");

    assert_eq!(std::fs::read_to_string(&path).expect("readable"), SHIPPED);
}

#[test]
fn an_entry_with_no_reason_never_reaches_the_file_because_the_daemon_would_refuse_it() {
    let path = temporary("reasonless.yaml");

    let refused = add(&Options {
        reason: "   ".into(),
        ..silencing(&path, "user|group|docker")
    })
    .expect_err("must not be accepted");

    assert!(refused.contains("reason"), "{refused}");
    assert_eq!(std::fs::read_to_string(&path).expect("readable"), SHIPPED);
}

#[test]
fn a_file_that_is_not_there_says_which_one_and_what_writes_it() {
    let refused =
        add(&silencing("/nonexistent/vigil/vigil.yaml", "a|b")).expect_err("must not be invented");

    assert!(
        refused.contains("/nonexistent/vigil/vigil.yaml"),
        "{refused}"
    );
    assert!(refused.contains("vigild configure"), "{refused}");
}

#[test]
fn what_the_file_holds_is_read_back_by_the_same_words_the_console_draws() {
    let path = temporary("list.yaml");
    assert!(
        list(&Options {
            path: path.clone(),
            ..Options::default()
        })
        .expect("reads")
        .said[0]
            .contains("silences nothing")
    );

    add(&silencing(&path, "user|group|docker")).expect("writes");

    let said = list(&Options {
        path,
        ..Options::default()
    })
    .expect("reads");
    assert!(said.said[0].contains("1 suppression(s)"), "{said:#?}");
    assert!(said.said[1].contains("user|group|docker"), "{said:#?}");
    assert!(said.said[1].contains("the staging api"), "{said:#?}");
}
