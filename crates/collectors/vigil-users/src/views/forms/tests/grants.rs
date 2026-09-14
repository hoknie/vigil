use serde_json::json;
use vigil_model::{AccountChange, Changing};

use super::harness::{change, form, put, refused_form, with_a_grant};
use crate::fixture::users;

#[test]
fn a_grant_that_lives_only_in_the_main_sudoers_file_is_not_changed_from_here() {
    let reading = users();

    let said = refused_form("sudo", &reading, Some("sudoer|%wheel"), Changing::Update);
    assert!(said.contains("/etc/sudoers"), "{said}");
    assert!(
        change(
            "sudo",
            &reading,
            Some("sudoer|%wheel"),
            Changing::Delete,
            None
        )
        .is_err()
    );
}

#[test]
fn a_grant_the_reading_hid_part_of_is_not_offered_for_editing() {
    let reading = users().with(
        "sudoer|ops",
        json!({
            "who": "ops",
            "group": false,
            "rules": [{
                "source": "/etc/sudoers.d/ops",
                "spec": "ALL=(ALL) /usr/bin/mysql -p[redacted]",
                "spec_redacted": true,
                "nopasswd": false,
                "all_commands": false,
            }],
            "nopasswd": false,
            "all_commands": false,
            "spec_redacted": true,
        }),
    );

    let said = refused_form("sudo", &reading, Some("sudoer|ops"), Changing::Update);
    assert!(said.contains("hides"), "{said}");
}

#[test]
fn a_grant_in_sudoers_d_is_written_from_its_rules_and_one_more() {
    let reading = with_a_grant();
    let mut grant = form("sudo", &reading, Some("sudoer|deploy"), Changing::Update);
    assert_eq!(grant.text("rule_1"), Some("ALL=(ALL) ALL"));
    assert_eq!(grant.text("new_rule"), Some(""));

    put(
        &mut grant,
        "new_rule",
        "ALL=(ALL) NOPASSWD: /usr/bin/systemctl",
    );
    assert_eq!(
        change(
            "sudo",
            &reading,
            Some("sudoer|deploy"),
            Changing::Update,
            Some(&grant)
        ),
        Ok(AccountChange::UpdateSudo {
            who: "deploy".into(),
            rules: vec![
                "ALL=(ALL) ALL".into(),
                "ALL=(ALL) NOPASSWD: /usr/bin/systemctl".into()
            ],
        })
    );

    put(&mut grant, "new_rule", "nonsense");
    assert!(
        change(
            "sudo",
            &reading,
            Some("sudoer|deploy"),
            Changing::Update,
            Some(&grant)
        )
        .expect_err("no =")
        .contains("not a sudo rule")
    );

    put(&mut grant, "new_rule", "");
    put(&mut grant, "rule_1", "");
    assert!(
        change(
            "sudo",
            &reading,
            Some("sudoer|deploy"),
            Changing::Update,
            Some(&grant)
        )
        .expect_err("emptied")
        .contains("press D")
    );
}

#[test]
fn a_rule_from_the_main_file_is_shown_beside_the_grant_and_not_offered_for_typing() {
    let reading = users().with(
        "sudoer|deploy",
        json!({
            "who": "deploy",
            "group": false,
            "rules": [
                {"source": "/etc/sudoers", "spec": "ALL=(ALL) /usr/bin/id", "spec_redacted": false},
                {"source": "/etc/sudoers.d/deploy", "spec": "ALL=(ALL) ALL", "spec_redacted": false},
            ],
            "spec_redacted": false,
        }),
    );

    let grant = form("sudo", &reading, Some("sudoer|deploy"), Changing::Update);
    let kept = grant
        .field("kept_1")
        .expect("the main file's rule is shown");
    assert!(!kept.editable());
    assert_eq!(grant.text("rule_1"), Some("ALL=(ALL) ALL"));
}
