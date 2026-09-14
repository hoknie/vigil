use vigil_model::{AccountChange, Changing};
use vigil_view::Entry;

use super::harness::{change, flip, form, put};
use crate::fixture::users;

#[test]
fn a_new_group_needs_a_name_this_host_accepts_and_nobody_has_taken() {
    let reading = users();
    let mut new = form("groups", &reading, None, Changing::Create);
    assert_eq!(new.caption, "NEW GROUP");
    assert_eq!(new.chosen("members"), Some(Vec::new()));

    for (name, why) in [
        ("", "needs a name"),
        ("Ops", "not a name"),
        ("wheel", "already"),
    ] {
        put(&mut new, "name", name);
        let said =
            change("groups", &reading, None, Changing::Create, Some(&new)).expect_err("refused");
        assert!(said.contains(why), "{name:?}: {said}");
    }

    put(&mut new, "name", "ops");
    flip(&mut new, "members", "deploy");
    assert_eq!(
        change("groups", &reading, None, Changing::Create, Some(&new)),
        Ok(AccountChange::CreateGroup {
            name: "ops".into(),
            members: vec!["deploy".into()],
        })
    );
}

#[test]
fn a_group_form_leaves_out_the_accounts_that_are_in_it_by_their_own_group() {
    let reading = users();

    let own = form("groups", &reading, Some("group|deploy"), Changing::Update);
    let members = own.field("members").expect("members");
    match &members.entry {
        Entry::Choices(choices) => assert!(!choices.iter().any(|one| one.name == "deploy")),
        other => panic!("{other:?}"),
    }
    assert!(
        members
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("deploy"))
    );

    let mut wheel = form("groups", &reading, Some("group|wheel"), Changing::Update);
    assert_eq!(wheel.chosen("members"), Some(vec!["deploy"]));
    flip(&mut wheel, "members", "contractor");
    put(&mut wheel, "rename", "admins");
    assert_eq!(
        change(
            "groups",
            &reading,
            Some("group|wheel"),
            Changing::Update,
            Some(&wheel)
        ),
        Ok(AccountChange::UpdateGroup {
            name: "wheel".into(),
            rename: Some("admins".into()),
            members: Some(vec!["contractor".into(), "deploy".into()]),
        })
    );
}

#[test]
fn the_group_root_is_not_deleted_and_another_group_is() {
    let reading = users();
    assert!(
        change(
            "groups",
            &reading,
            Some("group|root"),
            Changing::Delete,
            None
        )
        .is_err()
    );
    assert_eq!(
        change(
            "groups",
            &reading,
            Some("group|wheel"),
            Changing::Delete,
            None
        ),
        Ok(AccountChange::DeleteGroup {
            name: "wheel".into()
        })
    );
}
