use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::WhoCanLogIn;
use crate::fixture::users;

fn panes() -> Vec<Box<dyn Pane>> {
    WhoCanLogIn.panes()
}

#[test]
fn every_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    let reading = users();

    for pane in panes() {
        if !pane.shown(&reading) {
            continue;
        }
        conformance::run_all(pane.as_ref(), &reading);
    }
}

#[test]
fn a_pane_that_would_open_onto_nothing_is_not_offered_at_all() {
    let reading = users();
    let every = panes();
    let offered: Vec<&str> = every
        .iter()
        .filter(|pane| pane.shown(&reading))
        .map(|pane| pane.name())
        .collect();

    assert_eq!(
        offered,
        vec!["users", "groups", "sudo", "keys", "ssh users", "logged in"],
        "a tab that opens onto nothing teaches people to stop pressing the tabs"
    );
}

#[test]
fn an_account_is_shown_with_what_decides_whether_it_can_log_in() {
    let reading = users();
    let pane = &panes()[0];
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == "account|root")
        .expect("the sample has root");

    let cells = pane.cells(&reading, &row, Room::of(160));

    assert_eq!(cells[0].text, "root");
    assert_eq!(cells[1].text, "0");
    assert!(!cells[2].text.is_empty(), "the password state: {cells:?}");
}

#[test]
fn an_account_that_can_log_in_is_listed_before_every_account_that_cannot() {
    let reading = users();
    let pane = &panes()[0];

    let rows = pane.rows(&reading, &Showing::default());
    let could: Vec<bool> = rows
        .iter()
        .map(|row| crate::views::facts::could_log_in(&reading.items[&row.key]))
        .collect();
    let mut sorted = could.clone();
    sorted.sort_by(|left, right| right.cmp(left));

    assert_eq!(
        could, sorted,
        "what a reader is looking for on this screen comes first: {rows:?}"
    );
    assert!(could.first() == Some(&true));
}

#[test]
fn a_key_file_the_agent_was_refused_is_a_row_and_not_a_silence() {
    let reading = users();
    let keys = panes()
        .into_iter()
        .find(|pane| pane.name() == "keys")
        .expect("the keys pane");

    let rows = keys.rows(&reading, &Showing::default());

    assert!(
        rows.iter().any(|row| row.key.ends_with("|unreadable")),
        "a refusal reads as 'this account has no keys' unless the row says otherwise: {rows:?}"
    );
}

#[test]
fn what_the_detail_of_an_account_says_is_what_a_reader_can_act_on() {
    let reading = users();
    let pane = &panes()[0];
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key == "account|root")
        .expect("the sample has root");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("PASSWORD"), "{said}");
    assert!(said.contains("WHAT IT CAN REACH"), "{said}");
    assert!(
        said.contains("user|account|root"),
        "the key an operator puts in suppressions is the finding key: {said}"
    );
}

#[test]
fn a_sudo_grant_reaches_the_accounts_whose_own_grants_name_it_and_no_others() {
    let mut reading = users();
    reading.items.insert(
        "sudoer|contractor".into(),
        serde_json::json!({"who": "contractor", "rules": [
            {"source": "/etc/sudoers.d/contractor", "spec": "ALL=(root) /usr/bin/id"}
        ]}),
    );
    let sudo = panes()
        .into_iter()
        .find(|pane| pane.name() == "sudo")
        .expect("the sudo pane");

    for row in sudo.rows(&reading, &Showing::default()) {
        let who = reading.items[&row.key]["who"]
            .as_str()
            .expect("a grant names who");
        let said = format!("{:?}", sudo.detail(&reading, &row, 80));
        for (key, account) in reading
            .items
            .iter()
            .filter(|(key, _)| key.starts_with("account|"))
        {
            let name = account["name"].as_str().expect("an account has a name");
            let through_its_grants = crate::views::facts::sudo_for(&reading, name)
                .iter()
                .any(|(_, grant)| grant["who"].as_str() == Some(who));
            assert_eq!(
                said.contains(&format!("name: \"account\", value: \"{name}\" }}")),
                through_its_grants,
                "{who} and {key}: the grant is drawn from the group it names, and it must reach \
                 exactly the accounts whose own row says the grant reaches them: {said}"
            );
        }
    }
}

#[test]
fn a_pane_finds_the_row_a_finding_is_about_without_a_table_in_the_console() {
    let reading = users();
    let groups = panes()
        .into_iter()
        .find(|pane| pane.name() == "groups")
        .expect("the groups pane");

    assert!(groups.holds(&reading, "group|wheel"));
    assert!(!groups.holds(&reading, "account|root"));
}
