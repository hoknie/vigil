use serde_json::{Value, json};
use vigil_model::{Change, Finding};
use vigil_rules::RuleContext;

use crate::fixture::{walk, walked_file, watched_file};
use crate::rules::file_rules;

const TREE: &str = "/etc/pam.d";

fn judged(changes: &[Change]) -> Vec<Finding> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-18T12:00:00.000Z".into(),
        mint_event_id: &mut mint,
    };
    file_rules().judge(changes, &mut ctx)
}

fn arrived(path: &str) -> Change {
    Change::Added {
        key: format!("file|{path}"),
        after: walked_file(path, TREE, "0644"),
    }
}

fn left(path: &str) -> Change {
    Change::Removed {
        key: format!("file|{path}"),
        before: walked_file(path, TREE, "0644"),
    }
}

fn walk_went(before: Value, after: Value) -> Change {
    Change::Changed {
        key: format!("walk|{TREE}"),
        before,
        after,
    }
}

fn directory(path: &str) -> Value {
    let mut row = walked_file(path, TREE, "0755");
    row["type"] = json!("directory");
    row["sha256"] = Value::Null;
    row
}

#[test]
fn a_file_that_appears_inside_a_watched_directory_is_a_file_that_changed() {
    let findings = judged(&[
        arrived("/etc/pam.d/backdoor"),
        walk_went(walk(TREE, 3, true), walk(TREE, 4, true)),
    ]);

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].kind.as_str(), "file.changed");
    assert_eq!(findings[0].finding_key, "file|/etc/pam.d/backdoor");
    assert!(
        findings[0].title.contains("appeared under /etc/pam.d"),
        "{}",
        findings[0].title
    );
}

#[test]
fn a_file_that_leaves_a_watched_directory_is_a_file_that_changed_too() {
    let findings = judged(&[
        left("/etc/pam.d/su"),
        walk_went(walk(TREE, 4, true), walk(TREE, 3, true)),
    ]);

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].title.contains("is gone from /etc/pam.d"),
        "{}",
        findings[0].title
    );
    assert!(findings[0].after.is_none());
}

#[test]
fn a_directory_the_operator_has_just_added_to_the_list_is_learnt_and_not_reported_file_by_file() {
    let findings = judged(&[
        arrived("/etc/pam.d/sshd"),
        arrived("/etc/pam.d/su"),
        Change::Added {
            key: format!("walk|{TREE}"),
            after: walk(TREE, 2, true),
        },
    ]);

    assert!(
        findings.is_empty(),
        "an entry added to the watch list is the operator's own act, and what it brings into \
         view is the first reading of it: {findings:#?}"
    );
}

#[test]
fn a_directory_taken_off_the_list_says_nothing_about_the_files_it_no_longer_shows() {
    let findings = judged(&[
        left("/etc/pam.d/sshd"),
        Change::Removed {
            key: format!("walk|{TREE}"),
            before: walk(TREE, 1, true),
        },
    ]);

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn a_walk_cut_short_by_its_limit_is_never_read_as_files_that_came_and_went() {
    let mut cut = walked_file("/etc/pam.d/zz", TREE, "0644");
    cut["complete"] = json!(false);

    for changes in [
        vec![
            left("/etc/pam.d/zz"),
            walk_went(walk(TREE, 10, true), walk(TREE, 9, false)),
        ],
        vec![
            arrived("/etc/pam.d/aa"),
            walk_went(walk(TREE, 9, false), walk(TREE, 10, true)),
        ],
        vec![Change::Removed {
            key: "file|/etc/pam.d/zz".into(),
            before: cut.clone(),
        }],
        vec![Change::Added {
            key: "file|/etc/pam.d/zz".into(),
            after: cut.clone(),
        }],
    ] {
        assert!(
            judged(&changes).is_empty(),
            "a path past the limit of one reading and inside it at the next moved nowhere on \
             the host: {changes:#?}"
        );
    }
}

#[test]
fn a_filesystem_the_operator_stopped_walking_into_is_not_read_as_files_that_went() {
    let mut narrowed = walk(TREE, 1, true);
    narrowed["not_entered"] = json!(["/etc/pam.d/mnt"]);

    let findings = judged(&[
        left("/etc/pam.d/mnt/one"),
        walk_went(walk(TREE, 2, true), narrowed),
    ]);

    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn a_directory_that_appears_with_files_in_it_is_one_finding_that_counts_them() {
    let findings = judged(&[
        Change::Added {
            key: "file|/etc/pam.d/extra".into(),
            after: directory("/etc/pam.d/extra"),
        },
        arrived("/etc/pam.d/extra/one"),
        arrived("/etc/pam.d/extra/two"),
        arrived("/etc/pam.d/extra/deeper/three"),
        walk_went(walk(TREE, 3, true), walk(TREE, 7, true)),
    ]);

    assert_eq!(
        findings.len(),
        1,
        "one directory dropped with its contents is one act, and a finding per file is the \
         noise an operator silences the whole collector over: {findings:#?}"
    );
    assert_eq!(findings[0].subject.object, "directory");
    assert!(
        findings[0]
            .evidence
            .iter()
            .any(|evidence| evidence.value.starts_with("3 more path(s)")),
        "{:#?}",
        findings[0].evidence
    );
}

#[test]
fn a_file_named_in_the_list_that_appears_is_left_to_the_rule_it_always_belonged_to() {
    let absent = crate::fixture::watched_file_absent("/etc/hosts");
    let findings = judged(&[Change::Changed {
        key: "file|/etc/hosts".into(),
        before: absent,
        after: watched_file("/etc/hosts", "0644", "a9"),
    }]);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule.as_deref(), Some("file_changed"));
}

#[test]
fn a_file_named_in_the_list_the_operator_adds_is_a_first_reading_and_says_nothing() {
    let findings = judged(&[Change::Added {
        key: "file|/etc/sudoers.d/deploy".into(),
        after: watched_file("/etc/sudoers.d/deploy", "0440", "a9"),
    }]);

    assert!(findings.is_empty(), "{findings:#?}");
}
