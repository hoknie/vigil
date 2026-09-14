use vigil_model::{AccountChange, Changing};

use super::harness::{change, deploys_key, flip, form, put, with_a_grant};
use crate::fixture::users;

#[test]
fn an_account_form_starts_from_what_the_reading_says_about_the_account() {
    let reading = users();
    let deploy = form("users", &reading, Some("account|deploy"), Changing::Update);

    assert_eq!(deploy.caption, "EDIT THE ACCOUNT deploy");
    assert_eq!(deploy.text("name"), Some("deploy"));
    assert_eq!(deploy.text("uid"), Some("1000"));
    assert!(!deploy.field("uid").expect("uid").editable());
    assert_eq!(deploy.text("shell"), Some("/bin/bash"));
    assert_eq!(deploy.text("home"), Some("/home/deploy"));
    assert_eq!(deploy.text("comment"), Some(""));
    assert_eq!(deploy.switch("locked"), Some(false));
    assert_eq!(
        deploy.chosen("groups"),
        Some(vec!["wheel"]),
        "the group deploy is in by its own gid is not a membership usermod -G sets"
    );
    assert!(
        deploy
            .field("groups")
            .and_then(|field| field.hint.as_deref())
            .is_some_and(|hint| hint.contains("deploy")),
        "and the form says where it went"
    );
    assert!(!deploy.about.is_empty() && deploy.about[0].contains("usermod"));

    let locked = form(
        "users",
        &reading,
        Some("account|www-data"),
        Changing::Update,
    );
    assert_eq!(locked.switch("locked"), Some(true));
}

#[test]
fn an_account_whose_shadow_line_was_not_read_is_not_shown_as_known_to_be_unlocked() {
    let unread = form(
        "users",
        &users(),
        Some("account|svc-runner"),
        Changing::Update,
    );

    let hint = unread
        .field("locked")
        .and_then(|field| field.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("could not read"), "{hint}");
}

#[test]
fn an_account_change_carries_only_the_fields_a_person_changed() {
    let reading = users();
    let mut edited = form("users", &reading, Some("account|deploy"), Changing::Update);
    put(&mut edited, "shell", "/bin/sh");

    assert_eq!(
        change(
            "users",
            &reading,
            Some("account|deploy"),
            Changing::Update,
            Some(&edited)
        ),
        Ok(AccountChange::UpdateUser {
            name: "deploy".into(),
            shell: Some("/bin/sh".into()),
            home: None,
            comment: None,
            locked: None,
            groups: None,
        }),
        "a field sent back unchanged is a field the agent would rewrite on the host for nothing"
    );

    flip(&mut edited, "locked", "");
    flip(&mut edited, "groups", "wheel");
    match change(
        "users",
        &reading,
        Some("account|deploy"),
        Changing::Update,
        Some(&edited),
    ) {
        Ok(AccountChange::UpdateUser { locked, groups, .. }) => {
            assert_eq!(locked, Some(true));
            assert_eq!(groups, Some(Vec::new()));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_form_nobody_changed_is_refused_rather_than_sent() {
    let reading = with_a_grant();
    let key = deploys_key(&reading);

    for (list, at) in [
        ("users", "account|deploy"),
        ("groups", "group|wheel"),
        ("sudo", "sudoer|deploy"),
        ("keys", key.as_str()),
        ("ssh users", "account|deploy"),
    ] {
        let untouched = form(list, &reading, Some(at), Changing::Update);
        let said = change(list, &reading, Some(at), Changing::Update, Some(&untouched))
            .expect_err("nothing to send");
        assert!(said.contains("nothing was changed"), "{list}: {said}");
    }
}

#[test]
fn root_is_neither_deleted_nor_locked_from_this_console() {
    let reading = users();

    let said = change(
        "users",
        &reading,
        Some("account|root"),
        Changing::Delete,
        None,
    )
    .expect_err("root stays");
    assert!(said.contains("uid 0"), "{said}");

    let mut root = form("users", &reading, Some("account|root"), Changing::Update);
    flip(&mut root, "locked", "");
    assert!(
        change(
            "users",
            &reading,
            Some("account|root"),
            Changing::Update,
            Some(&root)
        )
        .is_err()
    );
}

#[test]
fn a_shell_or_a_home_that_is_not_an_absolute_path_is_refused_in_words() {
    let reading = users();
    for (field, value) in [("shell", "bash"), ("home", "/home/a:b"), ("comment", "a:b")] {
        let mut edited = form("users", &reading, Some("account|deploy"), Changing::Update);
        put(&mut edited, field, value);
        let said = change(
            "users",
            &reading,
            Some("account|deploy"),
            Changing::Update,
            Some(&edited),
        )
        .expect_err("refused");
        assert!(
            said.contains('/') || said.contains("colon"),
            "{field}: {said}"
        );
    }
}
