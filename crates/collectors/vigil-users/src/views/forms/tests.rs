use serde_json::json;
use vigil_model::{AccountChange, AccountObject, Changing, Snapshot};
use vigil_view::{Entry, Form, Pane, RowKey, Section};

use crate::fixture::{sudoer, users};
use crate::views::WhoCanLogIn;

const OTHER_KEY: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq other@laptop";

const DEPLOYS_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

fn pane(name: &str) -> Box<dyn Pane> {
    WhoCanLogIn
        .panes()
        .into_iter()
        .find(|pane| pane.name() == name)
        .unwrap_or_else(|| panic!("no {name} list"))
}

fn row(key: &str) -> RowKey {
    RowKey::of(key)
}

fn form(list: &str, reading: &Snapshot, key: Option<&str>, changing: Changing) -> Form {
    let key = key.map(row);
    pane(list)
        .form(reading, key.as_ref(), changing)
        .unwrap_or_else(|why| panic!("{list} {} has no form: {why}", changing.as_str()))
}

fn refused_form(list: &str, reading: &Snapshot, key: Option<&str>, changing: Changing) -> String {
    let key = key.map(row);
    pane(list)
        .form(reading, key.as_ref(), changing)
        .expect_err("the form must not open")
}

fn change(
    list: &str,
    reading: &Snapshot,
    key: Option<&str>,
    changing: Changing,
    form: Option<&Form>,
) -> Result<AccountChange, String> {
    let key = key.map(row);
    pane(list).change(reading, key.as_ref(), changing, form)
}

fn put(form: &mut Form, name: &str, value: &str) {
    form.field_mut(name)
        .unwrap_or_else(|| panic!("no field {name}"))
        .entry = Entry::Text(value.to_string());
}

fn flip(form: &mut Form, name: &str, choice: &str) {
    let field = form
        .field_mut(name)
        .unwrap_or_else(|| panic!("no field {name}"));
    match &mut field.entry {
        Entry::Choices(choices) => {
            let one = choices
                .iter_mut()
                .find(|one| one.name.starts_with(choice))
                .unwrap_or_else(|| panic!("no choice {choice}"));
            one.chosen = !one.chosen;
        }
        Entry::Switch(on) => *on = !*on,
        other => panic!("{name} is {other:?}"),
    }
}

fn with_a_grant() -> Snapshot {
    users().with(
        "sudoer|deploy",
        sudoer("deploy", "ALL=(ALL) ALL", false, true),
    )
}

fn deploys_key(reading: &Snapshot) -> String {
    reading
        .items
        .keys()
        .find(|key| key.starts_with("sshkey|deploy|"))
        .cloned()
        .expect("the sample lets deploy in by a key")
}

#[test]
fn every_list_offers_the_changes_of_what_it_lists_and_the_other_list_offers_none() {
    for (list, object) in [
        ("users", AccountObject::User),
        ("groups", AccountObject::Group),
        ("sudo", AccountObject::Sudo),
        ("keys", AccountObject::Key),
        ("ssh users", AccountObject::SshUser),
        ("logged in", AccountObject::Session),
    ] {
        let offers = pane(list).offers();
        assert_eq!(offers.changing, Some(object), "{list}");
        assert!(offers.marking, "{list}: a delete is aimed at marked rows");
        assert!(!offers.sorting, "{list} still keeps its own order");
    }

    let other = pane("other");
    assert_eq!(other.offers().changing, None);
    assert!(
        other
            .form(&users(), None, Changing::Create)
            .expect_err("nothing to change")
            .contains("read, not changed")
    );
}

#[test]
fn a_change_a_list_does_not_offer_is_refused_by_name() {
    let said = refused_form("users", &users(), None, Changing::Create);
    assert!(said.contains("does not create"), "{said}");
    assert!(said.contains("update, delete"), "{said}");

    let said = change(
        "logged in",
        &users(),
        Some("session|deploy|pts/0"),
        Changing::Update,
        None,
    )
    .expect_err("a session is not edited");
    assert!(said.contains("does not update"), "{said}");
}

#[test]
fn a_delete_is_asked_for_on_the_band_and_not_through_a_form() {
    let said = refused_form("users", &users(), Some("account|deploy"), Changing::Delete);
    assert!(said.contains("without a form"), "{said}");
}

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

#[test]
fn a_session_is_ended_by_its_row_and_a_row_about_login_records_is_not_a_session() {
    let reading = users().with(
        "session|ghost|pts/9",
        json!({"user": "ghost", "session_id": "", "pid": 0}),
    );

    assert_eq!(
        change(
            "logged in",
            &reading,
            Some("session|deploy|pts/0"),
            Changing::Delete,
            None
        ),
        Ok(AccountChange::DeleteSession {
            key: "session|deploy|pts/0".into()
        })
    );
    for (at, why) in [
        ("session-source|logind", "not a session"),
        ("session|ghost|pts/9", "nothing to end it by"),
    ] {
        let said =
            change("logged in", &reading, Some(at), Changing::Delete, None).expect_err("refused");
        assert!(said.contains(why), "{at}: {said}");
    }
}

#[test]
fn no_row_a_row_gone_since_or_a_row_of_another_list_is_refused_in_words() {
    let reading = users();

    for (at, why) in [
        (None, "no row"),
        (Some("account|nobody"), "not in the reading"),
        (Some("group|wheel"), "not an account"),
    ] {
        let said = refused_form("users", &reading, at, Changing::Update);
        assert!(said.contains(why), "{at:?}: {said}");
    }
}

#[test]
fn every_change_a_list_offers_can_be_reached_from_its_form_or_its_row() {
    let reading = with_a_grant();
    let key = deploys_key(&reading);
    let mut reached: Vec<(AccountObject, Changing)> = Vec::new();

    let mut ask = |list: &str, at: Option<&str>, changing: Changing, edit: &dyn Fn(&mut Form)| {
        let filled = match changing {
            Changing::Delete => None,
            _ => {
                let mut blank = form(list, &reading, at, changing);
                edit(&mut blank);
                Some(blank)
            }
        };
        let asked = change(list, &reading, at, changing, filled.as_ref())
            .unwrap_or_else(|why| panic!("{list} {}: {why}", changing.as_str()));
        assert_eq!(asked.changing(), changing, "{list}");
        reached.push((asked.object(), asked.changing()));
    };

    ask("users", Some("account|deploy"), Changing::Update, &|form| {
        put(form, "home", "/srv/deploy")
    });
    ask("users", Some("account|deploy"), Changing::Delete, &|_| {});
    ask("groups", None, Changing::Create, &|form| {
        put(form, "name", "ops")
    });
    ask("groups", Some("group|wheel"), Changing::Update, &|form| {
        put(form, "rename", "admins")
    });
    ask("groups", Some("group|wheel"), Changing::Delete, &|_| {});
    ask("sudo", Some("sudoer|deploy"), Changing::Update, &|form| {
        put(form, "rule_1", "ALL=(ALL) /usr/bin/id")
    });
    ask("sudo", Some("sudoer|deploy"), Changing::Delete, &|_| {});
    ask("keys", None, Changing::Create, &|form| {
        put(form, "account", "deploy");
        put(form, "line", OTHER_KEY);
    });
    ask("keys", Some(&key), Changing::Update, &|form| {
        put(form, "comment", "laptop")
    });
    ask("keys", Some(&key), Changing::Delete, &|_| {});
    ask("ssh users", None, Changing::Create, &|form| {
        put(form, "account", "root");
        put(form, "line", OTHER_KEY);
    });
    ask(
        "ssh users",
        Some("account|deploy"),
        Changing::Update,
        &|form| put(form, "add_key", OTHER_KEY),
    );
    ask(
        "ssh users",
        Some("account|deploy"),
        Changing::Delete,
        &|_| {},
    );
    ask(
        "logged in",
        Some("session|deploy|pts/0"),
        Changing::Delete,
        &|_| {},
    );

    for object in AccountObject::ALL {
        for changing in object.changings() {
            assert!(
                reached.contains(&(*object, *changing)),
                "{} offers to {} and no list leads there",
                object.as_str(),
                changing.as_str()
            );
        }
    }
}
