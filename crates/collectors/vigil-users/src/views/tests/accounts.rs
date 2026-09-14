use serde_json::{Value, json};

use super::host::{OLDER_AGENT, a_host_with_names_that_prefix_one_another as host};
use super::walks;
use crate::fixture::users;
use crate::parsers::PRIVILEGED_GROUPS;
use crate::types::Kind;
use crate::views::facts::{groups_for, keys_of, privileged, route_to_root, sessions_of};
use crate::views::fields::text;

fn keys<'a>(found: impl IntoIterator<Item = (&'a str, &'a Value)>) -> Vec<&'a str> {
    found.into_iter().map(|(key, _)| key).collect()
}

#[test]
fn the_host_the_lookups_are_proved_on_holds_every_case_a_lookup_could_get_wrong() {
    let reading = host();

    assert!(
        reading.items[OLDER_AGENT].get("groups").is_none(),
        "an account from an agent that did not list its groups must be in the proof, or the \
         walk it falls back to is never compared"
    );
    for key in [
        "account|deploy",
        "account|deploy2",
        "account|dep",
        "sshkey|dep|unreadable",
        "sudoer|%docker",
        "sudoer|%ghost",
        "sudoer|deploy2",
        "keyring|root|0x1234",
        "account",
        "accounting|x",
    ] {
        assert!(
            reading.items.contains_key(key),
            "{key}: without it one of the ways a range or a key lookup goes wrong is not tried"
        );
    }
    for user in ["deploy", "deploy2", "dep"] {
        assert!(
            !keys_of(&reading, user).collect::<Vec<_>>().is_empty(),
            "{user} needs a key row, or a range that spills into a longer name is not caught"
        );
        assert!(
            !sessions_of(&reading, user).is_empty(),
            "{user} needs a session row, or a range that spills into a longer name is not caught"
        );
    }
}

#[test]
fn the_groups_of_an_account_come_from_its_own_list_and_are_the_ones_a_walk_over_every_group_finds()
{
    for reading in [host(), users()] {
        for (key, account) in walks::every(&reading, Kind::Account) {
            let name = text(account, "name").unwrap_or_default();

            assert_eq!(
                keys(groups_for(&reading, account)),
                keys(walks::groups_of(&reading, name)),
                "{key}: the detail of an account looks each of its groups up by name instead of \
                 walking every group, and must list the same groups in the same order"
            );
        }
    }
}

#[test]
fn the_route_to_root_says_word_for_word_what_the_walk_over_every_group_and_grant_said() {
    for reading in [host(), users()] {
        for (key, account) in walks::every(&reading, Kind::Account) {
            assert_eq!(
                route_to_root(&reading, account),
                walks::route_to_root(&reading, account),
                "{key}: the cell drawn for every visible account is built from lookups, and a \
                 reader must not be able to tell it from the walk"
            );
        }
    }
}

#[test]
fn a_group_a_newer_agent_calls_privileged_is_named_in_the_route_of_an_account_that_lists_it() {
    let mut reading = host();
    reading.items.insert(
        "group|kvm".into(),
        json!({
            "name": "kvm",
            "gid": 36,
            "members": ["deploy"],
            "privileged": true,
            "privilege": "a reason this build has never heard of",
        }),
    );
    reading
        .items
        .get_mut("account|deploy")
        .and_then(|account| account["groups"].as_array_mut())
        .expect("deploy lists its groups")
        .push(json!("kvm"));

    let route = route_to_root(&reading, &reading.items["account|deploy"]);

    assert!(
        route.contains("kvm"),
        "a group is privileged because the reading says so, not because this build's table \
         happens to name it: {route}"
    );
}

#[test]
fn a_group_is_privileged_in_a_reading_exactly_when_its_name_is_in_the_table_the_view_looks_up() {
    let reading = users();

    for (_, group) in walks::every(&reading, Kind::Group) {
        let name = text(group, "name").expect("a group has a name");
        assert_eq!(
            privileged(group),
            PRIVILEGED_GROUPS.iter().any(|(listed, _)| *listed == name),
            "{name}: an account from an older agent has no list of its groups, and its route to \
             root finds privileged groups by the names in this table"
        );
    }
}

#[test]
fn the_keys_and_sessions_of_an_account_are_the_range_under_its_name_and_never_under_a_longer_one() {
    let reading = host();
    let mut names: Vec<&str> = walks::every(&reading, Kind::Account)
        .into_iter()
        .filter_map(|(_, account)| text(account, "name"))
        .collect();
    names.extend(["ghost", "unknown", "de", "?", ""]);

    for name in names {
        assert_eq!(
            keys(keys_of(&reading, name)),
            walks::keys_of(&reading, name),
            "{name}: the keys of one account are the range under sshkey|{name}|, and the \
             trailing bar is what keeps deploy2's keys out of deploy's"
        );
        assert_eq!(
            keys(sessions_of(&reading, name)),
            walks::sessions_of(&reading, name),
            "{name}: the sessions of one account are the range under session|{name}|, and the \
             trailing bar is what keeps deploy2's sessions out of deploy's"
        );
    }
}
