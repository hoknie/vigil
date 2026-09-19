use vigil_collect::{Collector, Health, name_of_user, outside_the_sample, this_account};
use vigil_model::{Golden, Shape, Snapshot};

use super::UsersCollector;

fn read() -> Snapshot {
    UsersCollector::new(|| "2026-09-19T12:00:00.000Z".to_string())
        .collect()
        .expect("Directory Services answers every account")
}

#[test]
fn the_account_running_this_test_is_read_from_directory_services_with_its_home_and_shell() {
    let me = this_account();
    let name = name_of_user(me).expect("this account has a name");
    let snapshot = read();

    let row = snapshot
        .items
        .get(&format!("account|{name}"))
        .unwrap_or_else(|| panic!("account|{name}"));
    assert_eq!(row["uid"], me);
    if let Some(home) = std::env::var("HOME").ok().filter(|_| me != 0) {
        assert_eq!(row["home"], home);
    }
    assert_eq!(
        row["shadow_readable"], true,
        "Directory Services says to every account whether a password is set"
    );
}

#[test]
fn root_and_the_accounts_the_system_runs_as_are_read_and_read_once() {
    let snapshot = read();

    let root = &snapshot.items["account|root"];
    assert_eq!(root["uid"], 0);
    assert!(
        snapshot
            .items
            .keys()
            .any(|key| key.starts_with("account|_")),
        "the service accounts of a Mac are named with an underscore"
    );
    let uids_of_root = snapshot
        .items
        .iter()
        .filter(|(key, item)| key.starts_with("account|") && item["uid"] == 0)
        .count();
    assert_eq!(
        uids_of_root, 1,
        "macOS answers root from its directory and again from /etc/passwd, and one account \
         read twice is not a second account on uid 0"
    );
}

#[test]
fn the_groups_that_may_act_as_root_on_a_mac_are_read_as_privileged() {
    let snapshot = read();

    for group in ["group|admin", "group|wheel"] {
        let row = &snapshot.items[group];
        assert_eq!(row["privileged"], true, "{group}: {row}");
        assert!(
            row["members"]
                .as_array()
                .is_some_and(|members| members.iter().any(|member| member == "root")),
            "{group}: {row}"
        );
    }
}

#[test]
fn the_logins_of_this_mac_are_read_from_the_record_macos_keeps_of_them() {
    let snapshot = read();

    let source = &snapshot.items["session-source|utmp"];
    assert_eq!(source["path"], "/var/run/utmpx");
    assert_eq!(source["present"], true);
    assert_eq!(source["read"], true);
    assert_eq!(
        source["sessions"].as_u64(),
        Some(
            snapshot
                .items
                .keys()
                .filter(|key| key.starts_with("session|"))
                .count() as u64
        )
    );
    assert!(
        !snapshot.items.contains_key("session-source|logind"),
        "there is no systemd-logind on a Mac, and a row saying so on every reading would be a \
         source that never answers"
    );
}

#[test]
fn a_reading_of_macos_has_the_shape_of_the_reading_the_rules_and_the_console_were_built_on() {
    let sample: Shape = serde_json::from_str(
        &Golden::snapshot("users")
            .held()
            .expect("the published shape of users"),
    )
    .expect("a shape");

    let drift = outside_the_sample(
        &Shape::of(&read()),
        &sample,
        &[
            "account.password_last_change_day",
            "account.password_max_age_days",
            "account.account_expires_day",
        ],
    );

    assert!(drift.is_empty(), "{drift:#?}");
}

#[test]
fn an_agent_not_running_as_root_says_the_grants_of_sudo_are_out_of_its_sight() {
    let health = UsersCollector::new(String::new).available();

    match this_account() {
        0 => assert_eq!(health, Health::Ok),
        _ => {
            let Health::Degraded(why) = health else {
                panic!("{health:?}")
            };
            assert!(why.contains("/private/etc/sudoers"), "{why}");
            assert!(why.contains("run as root"), "{why}");
        }
    }
}

#[test]
fn nothing_that_could_be_cracked_reaches_the_snapshot_of_this_mac() {
    let printed = serde_json::to_string(&read()).expect("serialises");

    for marker in ["$1$", "$5$", "$6$", "$y$", "$2b$", "********"] {
        assert!(!printed.contains(marker), "{marker}");
    }
}
