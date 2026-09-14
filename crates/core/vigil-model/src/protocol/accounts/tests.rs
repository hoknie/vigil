use super::*;

fn one_of_each() -> Vec<AccountChange> {
    vec![
        AccountChange::UpdateUser {
            name: "deploy".into(),
            shell: Some("/bin/sh".into()),
            home: None,
            comment: None,
            locked: Some(true),
            groups: None,
        },
        AccountChange::DeleteUser {
            name: "deploy".into(),
        },
        AccountChange::CreateGroup {
            name: "ops".into(),
            members: vec!["deploy".into()],
        },
        AccountChange::UpdateGroup {
            name: "ops".into(),
            rename: Some("operators".into()),
            members: None,
        },
        AccountChange::DeleteGroup { name: "ops".into() },
        AccountChange::UpdateSudo {
            who: "deploy".into(),
            rules: vec!["ALL=(ALL) ALL".into()],
        },
        AccountChange::DeleteSudo {
            who: "deploy".into(),
        },
        AccountChange::CreateKey {
            user: "deploy".into(),
            line: "ssh-ed25519 AAAA person@laptop".into(),
        },
        AccountChange::UpdateKey {
            user: "deploy".into(),
            fingerprint: "SHA256:abc".into(),
            options: Some("no-pty".into()),
            comment: None,
        },
        AccountChange::DeleteKey {
            user: "deploy".into(),
            fingerprint: "SHA256:abc".into(),
        },
        AccountChange::CreateSshUser {
            user: "contractor".into(),
            line: "ssh-ed25519 AAAA person@laptop".into(),
        },
        AccountChange::UpdateSshUser {
            user: "contractor".into(),
            removed: vec!["SHA256:abc".into()],
            added: Vec::new(),
        },
        AccountChange::DeleteSshUser {
            user: "contractor".into(),
        },
        AccountChange::DeleteSession {
            key: "session|deploy|pts/0".into(),
        },
    ]
}

#[test]
fn what_each_list_of_accounts_can_have_done_to_it_is_the_table_the_owner_decided() {
    use Changing::{Create, Delete, Update};

    assert_eq!(AccountObject::User.changings(), &[Update, Delete]);
    assert_eq!(AccountObject::Group.changings(), &[Create, Update, Delete]);
    assert_eq!(AccountObject::Sudo.changings(), &[Update, Delete]);
    assert_eq!(AccountObject::Key.changings(), &[Create, Update, Delete]);
    assert_eq!(
        AccountObject::SshUser.changings(),
        &[Create, Update, Delete]
    );
    assert_eq!(
        AccountObject::Session.changings(),
        &[Delete],
        "a session is ended and nothing else: there is nothing in one to edit, and a \
         session is not created from a console"
    );
}

#[test]
fn every_change_the_protocol_can_carry_is_one_its_object_offers() {
    let every = one_of_each();

    for change in &every {
        assert!(
            change.object().offers(change.changing()),
            "{change:?} is a change the table of what each list offers does not hold"
        );
    }
    for object in AccountObject::ALL {
        for changing in object.changings() {
            assert!(
                every
                    .iter()
                    .any(|change| change.object() == *object && change.changing() == *changing),
                "{} offers to {} and the protocol has no way to ask for it",
                object.as_str(),
                changing.as_str()
            );
        }
    }
}

#[test]
fn a_change_round_trips_through_the_wire_under_the_name_of_what_it_does() {
    for change in one_of_each() {
        let line = serde_json::to_string(&change).expect("serialises");
        assert!(line.contains("\"change\":\""), "{line}");
        assert_eq!(
            serde_json::from_str::<AccountChange>(&line).expect("reads back"),
            change
        );
    }
    assert_eq!(
        serde_json::to_string(&AccountChange::DeleteUser {
            name: "deploy".into()
        })
        .expect("serialises"),
        "{\"change\":\"delete_user\",\"name\":\"deploy\"}"
    );
}

#[test]
fn a_change_this_build_has_not_heard_of_is_refused_rather_than_guessed_at() {
    for line in [
        "{\"change\":\"create_user\",\"name\":\"eve\"}",
        "{\"change\":\"update_session\",\"key\":\"session|deploy|pts/0\"}",
        "{\"change\":\"run\",\"command\":\"id\"}",
    ] {
        assert!(
            serde_json::from_str::<AccountChange>(line).is_err(),
            "{line} is not in the table of what the console may ask for"
        );
    }
}

#[test]
fn every_change_names_the_row_of_the_reading_it_is_about() {
    for change in one_of_each() {
        let key = change.key();
        let family = match change.object() {
            AccountObject::User | AccountObject::SshUser => "account|",
            AccountObject::Group => "group|",
            AccountObject::Sudo => "sudoer|",
            AccountObject::Key => "sshkey|",
            AccountObject::Session => "session|",
        };
        assert!(key.starts_with(family), "{change:?} names {key}");
        assert!(!change.said().is_empty());
    }
}

#[test]
fn an_update_that_changes_nothing_says_so_and_a_delete_is_never_empty() {
    let nothing = AccountChange::UpdateUser {
        name: "deploy".into(),
        shell: None,
        home: None,
        comment: None,
        locked: None,
        groups: None,
    };

    assert!(nothing.is_empty());
    assert!(nothing.said().contains("nothing"), "{}", nothing.said());
    for change in one_of_each() {
        assert!(!change.is_empty(), "{change:?}");
    }
}

#[test]
fn every_object_says_in_words_what_deleting_one_does() {
    for object in AccountObject::ALL {
        assert!(!object.deleting().is_empty());
        assert!(!object.named().is_empty());
    }
    assert!(
        AccountObject::User
            .deleting()
            .contains("home directory stays"),
        "the owner decided the home stays, and the band is where the reader learns that"
    );
}

#[test]
fn a_report_names_every_row_the_agent_left_alone_and_counts_both_halves() {
    let every = one_of_each();
    let report = ChangeReport {
        acted_at: "2026-09-14T10:00:00.000Z".into(),
        changed: vec![
            Changed::done(&every[0], "usermod finished"),
            Changed::refused(&every[1], "uid 0 is not deleted"),
        ],
    };

    assert_eq!(report.done(), 1);
    assert_eq!(report.refused(), 1);
    assert_eq!(report.changed[1].key, "account|deploy");
    assert_eq!(report.changed[1].object, AccountObject::User);
    assert_eq!(report.changed[1].changing, Changing::Delete);
    assert_eq!(
        Changed::done(&every[7], "added")
            .keyed("sshkey|deploy|SHA256:new")
            .key,
        "sshkey|deploy|SHA256:new"
    );
}
