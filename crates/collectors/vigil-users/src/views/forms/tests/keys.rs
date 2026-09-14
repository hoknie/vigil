use vigil_model::{AccountChange, Changing};

use super::harness::{DEPLOYS_KEY, OTHER_KEY, change, deploys_key, form, put, refused_form};
use crate::fixture::users;

#[test]
fn a_new_key_is_checked_against_the_account_and_against_the_line() {
    let reading = users();
    let key = deploys_key(&reading);

    let prefilled = form("keys", &reading, Some(&key), Changing::Create);
    assert_eq!(prefilled.text("account"), Some("deploy"));

    let mut new = form("keys", &reading, None, Changing::Create);
    assert_eq!(new.text("account"), Some(""));
    for (account, line, why) in [
        ("nobody", OTHER_KEY, "no account"),
        ("deploy", "hello", "no key"),
        ("deploy", DEPLOYS_KEY, "already"),
    ] {
        put(&mut new, "account", account);
        put(&mut new, "line", line);
        let said =
            change("keys", &reading, None, Changing::Create, Some(&new)).expect_err("refused");
        assert!(said.contains(why), "{account} {line}: {said}");
    }

    put(&mut new, "line", OTHER_KEY);
    assert_eq!(
        change("keys", &reading, None, Changing::Create, Some(&new)),
        Ok(AccountChange::CreateKey {
            user: "deploy".into(),
            line: OTHER_KEY.into(),
        })
    );
}

#[test]
fn a_key_file_the_agent_could_not_read_refuses_every_change_in_words() {
    let reading = users();
    let unread = Some("sshkey|backup|unreadable");

    for changing in [Changing::Create, Changing::Update] {
        let said = refused_form("keys", &reading, unread, changing);
        assert!(said.contains("could not be read"), "{said}");
    }
    assert!(
        change("keys", &reading, unread, Changing::Delete, None)
            .expect_err("unread")
            .contains("could not be read")
    );
}

#[test]
fn a_key_edit_sends_its_options_and_leaves_the_key_itself_alone() {
    let reading = users();
    let key = deploys_key(&reading);
    let mut edited = form("keys", &reading, Some(&key), Changing::Update);
    assert!(!edited.field("fingerprint").expect("fingerprint").editable());
    put(&mut edited, "options", "no-pty");

    match change(
        "keys",
        &reading,
        Some(&key),
        Changing::Update,
        Some(&edited),
    ) {
        Ok(AccountChange::UpdateKey {
            user,
            options,
            comment,
            ..
        }) => {
            assert_eq!(user, "deploy");
            assert_eq!(options.as_deref(), Some("no-pty"));
            assert_eq!(comment, None);
        }
        other => panic!("{other:?}"),
    }
}
