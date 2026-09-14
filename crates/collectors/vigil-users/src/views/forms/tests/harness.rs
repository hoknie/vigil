use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Entry, Form, Pane, RowKey, Section};

use crate::fixture::{sudoer, users};
use crate::views::WhoCanLogIn;

pub(super) const OTHER_KEY: &str =
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRq other@laptop";

pub(super) const DEPLOYS_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

pub(super) fn pane(name: &str) -> Box<dyn Pane> {
    WhoCanLogIn
        .panes()
        .into_iter()
        .find(|pane| pane.name() == name)
        .unwrap_or_else(|| panic!("no {name} list"))
}

pub(super) fn row(key: &str) -> RowKey {
    RowKey::of(key)
}

pub(super) fn form(list: &str, reading: &Snapshot, key: Option<&str>, changing: Changing) -> Form {
    let key = key.map(row);
    pane(list)
        .form(reading, key.as_ref(), changing)
        .unwrap_or_else(|why| panic!("{list} {} has no form: {why}", changing.as_str()))
}

pub(super) fn refused_form(
    list: &str,
    reading: &Snapshot,
    key: Option<&str>,
    changing: Changing,
) -> String {
    let key = key.map(row);
    pane(list)
        .form(reading, key.as_ref(), changing)
        .expect_err("the form must not open")
}

pub(super) fn change(
    list: &str,
    reading: &Snapshot,
    key: Option<&str>,
    changing: Changing,
    form: Option<&Form>,
) -> Result<AccountChange, String> {
    let key = key.map(row);
    pane(list).change(reading, key.as_ref(), changing, form)
}

pub(super) fn put(form: &mut Form, name: &str, value: &str) {
    form.field_mut(name)
        .unwrap_or_else(|| panic!("no field {name}"))
        .entry = Entry::Text(value.to_string());
}

pub(super) fn flip(form: &mut Form, name: &str, choice: &str) {
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

pub(super) fn with_a_grant() -> Snapshot {
    users().with(
        "sudoer|deploy",
        sudoer("deploy", "ALL=(ALL) ALL", false, true),
    )
}

pub(super) fn deploys_key(reading: &Snapshot) -> String {
    reading
        .items
        .keys()
        .find(|key| key.starts_with("sshkey|deploy|"))
        .cloned()
        .expect("the sample lets deploy in by a key")
}
