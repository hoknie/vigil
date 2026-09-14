use vigil_model::{AccountChange, Changing};

use super::harness::{OTHER_KEY, change, flip, form, put};
use crate::fixture::users;

#[test]
fn an_ssh_user_is_made_only_of_an_account_no_key_lets_in_yet() {
    let reading = users();
    let mut new = form("ssh users", &reading, None, Changing::Create);

    put(&mut new, "account", "deploy");
    put(&mut new, "line", OTHER_KEY);
    assert!(
        change("ssh users", &reading, None, Changing::Create, Some(&new))
            .expect_err("deploy has a key")
            .contains("already")
    );

    put(&mut new, "account", "root");
    assert_eq!(
        change("ssh users", &reading, None, Changing::Create, Some(&new)),
        Ok(AccountChange::CreateSshUser {
            user: "root".into(),
            line: OTHER_KEY.into(),
        })
    );
}

#[test]
fn the_keys_of_an_ssh_user_left_unchosen_go_and_a_line_typed_is_added() {
    let reading = users();
    let mut keys = form(
        "ssh users",
        &reading,
        Some("account|contractor"),
        Changing::Update,
    );
    let fingerprint = keys.chosen("keys").expect("keys")[0].to_string();

    flip(&mut keys, "keys", &fingerprint);
    put(&mut keys, "add_key", OTHER_KEY);

    assert_eq!(
        change(
            "ssh users",
            &reading,
            Some("account|contractor"),
            Changing::Update,
            Some(&keys)
        ),
        Ok(AccountChange::UpdateSshUser {
            user: "contractor".into(),
            removed: vec![fingerprint],
            added: vec![OTHER_KEY.into()],
        })
    );
    assert_eq!(
        change(
            "ssh users",
            &reading,
            Some("account|deploy"),
            Changing::Delete,
            None
        ),
        Ok(AccountChange::DeleteSshUser {
            user: "deploy".into()
        })
    );
}
