use serde_json::Value;
use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Choice, Field, Form, RowKey};

use crate::helpers::{absolute_path, changed_text, chosen, expect_kind, one_line, row_of};
use crate::types::Kind;
use crate::views::fields::{members, objects, text};

const ABOUT: &str = "The agent runs usermod on this host for the fields changed below, and \
                     leaves everything else as it is.";

const ROOT_STAYS: &str = "uid 0 is root: this console neither deletes nor locks it";

fn account<'a>(
    reading: &'a Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<&'a Value, String> {
    let (key, item) = row_of(reading, row, changing)?;
    expect_kind(key, Kind::Account, "an account")?;
    Ok(item)
}

fn is_root(item: &Value) -> bool {
    item.get("uid").and_then(Value::as_u64) == Some(0)
}

pub fn update_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let item = account(reading, row, Changing::Update)?;
    let name = text(item, "name").unwrap_or("?");
    let gid = item.get("gid").and_then(Value::as_u64);

    let mut own: Option<&str> = None;
    let mut choices: Vec<Choice> = Vec::new();
    for (_, group) in objects(reading, Kind::Group) {
        let Some(group_name) = text(group, "name") else {
            continue;
        };
        if gid.is_some() && group.get("gid").and_then(Value::as_u64) == gid {
            own = Some(group_name);
            continue;
        }
        choices.push(Choice::of(
            group_name,
            members(group).any(|member| member == name),
        ));
    }
    let groups = Field::choices("groups", "groups", choices);
    let groups = match own {
        Some(own) => groups.hinted(format!(
            "its own group {own} comes with the account and is not listed here"
        )),
        None => groups,
    };

    let (locked, hint) = locked(item);

    Ok(Form::new(format!("EDIT THE ACCOUNT {name}"))
        .saying(ABOUT)
        .with(Field::fixed("name", "name", name))
        .with(Field::fixed(
            "uid",
            "uid",
            item.get("uid")
                .and_then(Value::as_u64)
                .map_or_else(|| "?".to_string(), |uid| uid.to_string()),
        ))
        .with(Field::text(
            "shell",
            "shell",
            text(item, "shell").unwrap_or_default(),
        ))
        .with(
            Field::text("home", "home", text(item, "home").unwrap_or_default())
                .hinted("the field changes; the files stay where they are"),
        )
        .with(
            Field::text("comment", "comment", "")
                .hinted("not in the reading; left empty, it is left as it is"),
        )
        .with(Field::switch("locked", "locked", locked).hinted(hint))
        .with(groups))
}

fn locked(item: &Value) -> (bool, String) {
    if item.get("shadow_readable").and_then(Value::as_bool) != Some(true) {
        return (
            false,
            "the agent could not read this account's line in /etc/shadow, so whether it is \
             locked is not known; switched on, it is locked"
                .to_string(),
        );
    }
    match text(item, "password") {
        Some("locked") => (
            true,
            "the password starts with !; switched off, usermod -U takes the ! away".to_string(),
        ),
        Some("disabled") => (
            false,
            "the password is * and lets nobody in already; a key still can".to_string(),
        ),
        _ => (
            false,
            "usermod -L puts ! in front of the password; a key still lets it in".to_string(),
        ),
    }
}

pub fn update(
    reading: &Snapshot,
    row: Option<&RowKey>,
    form: &Form,
) -> Result<AccountChange, String> {
    let item = account(reading, row, Changing::Update)?;

    let shell = changed_text(form, "shell");
    if let Some(shell) = &shell {
        absolute_path("the shell", shell)?;
    }
    let home = changed_text(form, "home");
    if let Some(home) = &home {
        absolute_path("the home directory", home)?;
    }
    let comment = changed_text(form, "comment");
    if let Some(comment) = &comment {
        one_line("the comment", comment)?;
        if comment.contains(':') {
            return Err(
                "the comment cannot hold a colon: /etc/passwd separates its fields with colons"
                    .to_string(),
            );
        }
    }
    let locked = match form.changed("locked") {
        true => form.switch("locked"),
        false => None,
    };
    if locked == Some(true) && is_root(item) {
        return Err(ROOT_STAYS.to_string());
    }
    let groups = match form.changed("groups") {
        true => Some(chosen(form, "groups")),
        false => None,
    };

    Ok(AccountChange::UpdateUser {
        name: text(item, "name").unwrap_or_default().to_string(),
        shell,
        home,
        comment,
        locked,
        groups,
    })
}

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    let item = account(reading, row, Changing::Delete)?;
    if is_root(item) {
        return Err(ROOT_STAYS.to_string());
    }
    Ok(AccountChange::DeleteUser {
        name: text(item, "name").unwrap_or_default().to_string(),
    })
}
