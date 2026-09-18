use vigil_config::silenced;

use super::{NEXT_ROUND, Options, add, list, remove, take_out};

const SHIPPED: &str = "state_dir: /var/lib/vigil\nsuppressions: []\nreporters: []\n";

fn temporary(name: &str) -> String {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-silence-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos()
                + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
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
        said.said.iter().any(|line| line.contains(NEXT_ROUND)),
        "an entry nobody is told when it applies, is an \
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
    assert!(said.said[2].contains("user|group|docker"), "{said:#?}");
    assert!(said.said[2].contains("the staging api"), "{said:#?}");
}

const POINTING: &str =
    "state_dir: /var/lib/vigil\nsuppressions_path: suppressions\nreporters: []\n";

fn pointing(name: &str) -> (String, std::path::PathBuf) {
    let path = temporary(name);
    std::fs::write(&path, POINTING).expect("writes");
    let directory = std::path::Path::new(&path)
        .parent()
        .expect("a directory")
        .join("suppressions");
    (path, directory)
}

#[test]
fn a_configuration_that_points_at_a_directory_is_never_written_to_itself() {
    let (path, directory) = pointing("apart.yaml");

    add(&silencing(&path, "port.listen|tcp|0.0.0.0:4444")).expect("writes");

    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        POINTING,
        "the file `vigild configure --force` writes again must hold nothing it would lose"
    );
    let console = std::fs::read_to_string(directory.join(vigil_config::CONSOLE_FILE))
        .expect("the console's own file");
    assert_eq!(
        vigil_config::apart(&console)
            .expect("the daemon would read it")
            .len(),
        1,
        "{console}"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&directory)
            .expect("made")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700, "mode was {:o}", mode & 0o777);
    }
}

#[test]
fn a_file_asked_for_by_name_is_where_the_entry_goes_and_the_console_file_is_left_alone() {
    let (path, directory) = pointing("named.yaml");

    add(&Options {
        file: Some("deploy".into()),
        ..silencing(&path, "user|group|docker")
    })
    .expect("writes");

    assert!(directory.join("deploy.yaml").exists());
    assert!(!directory.join(vigil_config::CONSOLE_FILE).exists());
}

#[test]
fn an_entry_already_written_in_another_file_of_the_directory_is_not_written_twice() {
    let (path, directory) = pointing("elsewhere.yaml");
    std::fs::create_dir_all(&directory).expect("a directory");
    std::fs::write(
        directory.join("10-by-hand.yaml"),
        "suppressions:\n  - finding_key: \"user|group|docker\"\n    reason: ours\n",
    )
    .expect("writes");

    let done = add(&silencing(&path, "user|group|docker")).expect("reads");

    assert_eq!(done.entries, 0, "{:#?}", done.said);
    assert!(done.said[0].contains("10-by-hand.yaml"), "{:#?}", done.said);
    assert!(!directory.join(vigil_config::CONSOLE_FILE).exists());
}

#[test]
fn an_object_is_reported_again_whichever_file_it_was_silenced_in() {
    let (path, directory) = pointing("everywhere.yaml");
    std::fs::create_dir_all(&directory).expect("a directory");
    std::fs::write(
        directory.join("10-by-hand.yaml"),
        "suppressions:\n  - finding_key: \"user|group|docker\"\n    reason: ours\n",
    )
    .expect("writes");
    add(&Options {
        kind: Some("user.group.privileged_member_added".into()),
        ..silencing(&path, "user|group|docker")
    })
    .expect("writes");

    let done = remove(&Options {
        keys: vec!["user|group|docker".into()],
        path: path.clone(),
        ..Options::default()
    })
    .expect("writes");

    assert_eq!(done.entries, 2, "{:#?}", done.said);
    assert!(
        list(&Options {
            path,
            ..Options::default()
        })
        .expect("reads")
        .said[0]
            .contains("silences nothing")
    );
}

#[test]
fn one_entry_is_taken_out_of_the_file_that_holds_it_and_its_neighbours_stay() {
    let (path, directory) = pointing("one.yaml");
    add(&silencing(&path, "user|group|docker")).expect("writes");
    add(&silencing(&path, "user|group|sudo")).expect("writes");
    let console = directory.join(vigil_config::CONSOLE_FILE);

    take_out(
        &Options {
            path: path.clone(),
            ..Options::default()
        },
        &console,
        &vigil_config::Suppression {
            finding_key: Some("user|group|docker".into()),
            reason: "any".into(),
            ..vigil_config::Suppression::default()
        },
    )
    .expect("writes");

    let left = std::fs::read_to_string(&console).expect("readable");
    assert!(!left.contains("docker"), "{left}");
    assert!(left.contains("user|group|sudo"), "{left}");
}

#[test]
fn a_file_that_is_not_one_of_the_sources_is_not_edited_whatever_the_console_asks() {
    let (path, _) = pointing("stranger.yaml");
    let stranger = std::env::temp_dir().join("vigil-not-a-source.yaml");

    let refused = take_out(
        &Options {
            path,
            ..Options::default()
        },
        &stranger,
        &vigil_config::Suppression {
            finding_key: Some("a|b".into()),
            reason: "any".into(),
            ..vigil_config::Suppression::default()
        },
    )
    .expect_err("must not be touched");

    assert!(refused.contains("not a file"), "{refused}");
}

#[test]
fn a_file_asked_for_by_name_with_no_directory_to_put_it_in_is_refused_rather_than_guessed() {
    let path = temporary("nowhere.yaml");

    let refused = add(&Options {
        file: Some("deploy".into()),
        ..silencing(&path, "user|group|docker")
    })
    .expect_err("must not be accepted");

    assert!(refused.contains("suppressions_path"), "{refused}");
    assert_eq!(std::fs::read_to_string(&path).expect("readable"), SHIPPED);
}
