use serde_json::json;
use vigil_model::{AccountChange, Snapshot};
use vigil_users::fixture::users;

use super::harness::{disk, planned};
use crate::accounts::plans::plan;
use crate::accounts::step::Step;

fn with_sudoers_d() -> Snapshot {
    let mut reading = users();
    reading.items.insert(
        "sudoer|deploy".into(),
        json!({"who": "deploy", "rules": [
            {"source": "/etc/sudoers.d/deploy", "spec": "ALL=(ALL) ALL"},
            {"source": "/etc/sudoers.d/extra", "spec": "ALL=(root) /usr/bin/id"},
            {"source": "/etc/sudoers", "spec": "ALL=(ALL) ALL"}
        ]}),
    );
    reading
}

#[test]
fn a_grant_rewritten_goes_through_visudo_in_the_first_file_that_holds_it_and_leaves_etc_sudoers_alone()
 {
    let steps = planned(
        AccountChange::UpdateSudo {
            who: "deploy".into(),
            rules: vec!["ALL=(root) /usr/bin/systemctl restart app".into()],
        },
        &with_sudoers_d(),
        &[
            ("/etc/sudoers.d/deploy", "# app\ndeploy ALL=(ALL) ALL\n"),
            (
                "/etc/sudoers.d/extra",
                "deploy ALL=(root) /usr/bin/id\n%ops ALL=(ALL) ALL\n",
            ),
        ],
    );

    assert_eq!(
        steps,
        vec![
            Step::Sudoers {
                path: "/etc/sudoers.d/deploy".into(),
                text: "# app\ndeploy ALL=(root) /usr/bin/systemctl restart app\n".into()
            },
            Step::Sudoers {
                path: "/etc/sudoers.d/extra".into(),
                text: "%ops ALL=(ALL) ALL\n".into()
            },
        ]
    );
}

#[test]
fn a_grant_taken_away_removes_a_file_left_with_nothing_but_comments() {
    let steps = planned(
        AccountChange::DeleteSudo {
            who: "deploy".into(),
        },
        &with_sudoers_d(),
        &[
            ("/etc/sudoers.d/deploy", "# app\ndeploy ALL=(ALL) ALL\n"),
            (
                "/etc/sudoers.d/extra",
                "deploy ALL=(root) /usr/bin/id\n%ops ALL=(ALL) ALL\n",
            ),
        ],
    );

    assert_eq!(
        steps[0],
        Step::Remove {
            path: "/etc/sudoers.d/deploy".into(),
            holder: None
        }
    );
    assert!(matches!(&steps[1], Step::Sudoers { text, .. } if text == "%ops ALL=(ALL) ALL\n"));
}

#[test]
fn files_that_no_longer_hold_the_grant_the_reading_saw_change_nothing() {
    let disk = disk(&[
        ("/etc/sudoers.d/deploy", "# emptied by hand\n"),
        ("/etc/sudoers.d/extra", "%ops ALL=(ALL) ALL\n"),
    ]);

    let complaint = plan(
        &AccountChange::DeleteSudo {
            who: "deploy".into(),
        },
        &with_sudoers_d(),
        &|path, _| Ok(disk.get(path).cloned()),
    )
    .expect_err("stale");

    assert!(complaint.contains("older than the files"), "{complaint}");
}
