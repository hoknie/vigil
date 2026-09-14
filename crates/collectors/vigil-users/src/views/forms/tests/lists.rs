use vigil_model::{AccountObject, Changing};
use vigil_view::Form;

use super::harness::{OTHER_KEY, change, deploys_key, form, pane, put, refused_form, with_a_grant};
use crate::fixture::users;

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
