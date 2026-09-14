use vigil_model::AccountChange;
use vigil_users::fixture::users;

use super::harness::{arguments, planned};
use crate::accounts::utility::Utility;

#[test]
fn an_account_is_changed_by_one_usermod_with_each_value_its_own_argument_and_the_name_last() {
    let steps = planned(
        AccountChange::UpdateUser {
            name: "deploy".into(),
            shell: Some("/bin/sh".into()),
            home: Some("/srv/deploy".into()),
            comment: Some("Deploy; rm -rf /".into()),
            locked: Some(true),
            groups: Some(vec!["wheel".into(), "users".into()]),
        },
        &users(),
        &[],
    );

    assert_eq!(
        arguments(&steps),
        vec![(
            Utility::Usermod,
            [
                "-s",
                "/bin/sh",
                "-d",
                "/srv/deploy",
                "-c",
                "Deploy; rm -rf /",
                "-L",
                "-G",
                "wheel,users",
                "deploy"
            ]
            .map(String::from)
            .to_vec()
        )],
        "no -m: the home field changes, the files stay where they are; and a comment \
         that reads like a command is one argument that no shell ever sees"
    );
}

#[test]
fn taking_every_supplementary_group_away_is_an_empty_list_and_not_a_missing_flag() {
    let steps = planned(
        AccountChange::UpdateUser {
            name: "deploy".into(),
            shell: None,
            home: None,
            comment: None,
            locked: Some(false),
            groups: Some(Vec::new()),
        },
        &users(),
        &[],
    );

    assert_eq!(
        arguments(&steps)[0].1,
        ["-U", "-G", "", "deploy"].map(String::from).to_vec()
    );
}

#[test]
fn an_account_is_deleted_without_its_home_directory() {
    let steps = planned(
        AccountChange::DeleteUser {
            name: "contractor".into(),
        },
        &users(),
        &[],
    );

    assert_eq!(
        arguments(&steps),
        vec![(Utility::Userdel, vec!["contractor".to_string()])],
        "the owner decided the home stays: userdel without -r"
    );
}

#[test]
fn a_group_is_created_then_given_its_members_and_renamed_only_after_they_are_set() {
    assert_eq!(
        arguments(&planned(
            AccountChange::CreateGroup {
                name: "ops".into(),
                members: vec!["deploy".into(), "contractor".into()]
            },
            &users(),
            &[]
        )),
        vec![
            (Utility::Groupadd, vec!["ops".to_string()]),
            (
                Utility::Gpasswd,
                ["-M", "deploy,contractor", "ops"]
                    .map(String::from)
                    .to_vec()
            )
        ]
    );
    assert_eq!(
        arguments(&planned(
            AccountChange::UpdateGroup {
                name: "wheel".into(),
                rename: Some("admins".into()),
                members: Some(Vec::new())
            },
            &users(),
            &[]
        )),
        vec![
            (
                Utility::Gpasswd,
                ["-M", "", "wheel"].map(String::from).to_vec()
            ),
            (
                Utility::Groupmod,
                ["-n", "admins", "wheel"].map(String::from).to_vec()
            )
        ]
    );
}
