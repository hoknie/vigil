use serde_json::Value;
use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Choice, Field, Form, RowKey};

use crate::helpers::{changed_text, chosen, expect_kind, name_accepted, row_of, typed};
use crate::types::Kind;
use crate::views::fields::{members, objects, text};

fn group<'a>(
    reading: &'a Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<&'a Value, String> {
    let (key, item) = row_of(reading, row, changing)?;
    expect_kind(key, Kind::Group, "a group")?;
    Ok(item)
}

fn accounts(reading: &Snapshot) -> Vec<(&str, Option<u64>)> {
    let mut names: Vec<(&str, Option<u64>)> = objects(reading, Kind::Account)
        .filter_map(|(_, item)| {
            text(item, "name").map(|name| (name, item.get("gid").and_then(Value::as_u64)))
        })
        .collect();
    names.sort_unstable();
    names
}

fn free_name(reading: &Snapshot, name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("a group needs a name: type one into the name field".to_string());
    }
    name_accepted("the group name", name)?;
    match reading.items.contains_key(&format!("group|{name}")) {
        true => Err(format!("there is already a group {name} on this host")),
        false => Ok(()),
    }
}

pub fn create_form(reading: &Snapshot) -> Form {
    Form::new("NEW GROUP")
        .saying("The agent runs groupadd for the name, then gpasswd -M for the members chosen.")
        .with(
            Field::text("name", "name", "")
                .hinted("lowercase letters, digits, _ . or -, starting with a letter or _"),
        )
        .with(Field::choices(
            "members",
            "members",
            accounts(reading)
                .into_iter()
                .map(|(name, _)| Choice::of(name, false))
                .collect(),
        ))
}

pub fn create(reading: &Snapshot, form: &Form) -> Result<AccountChange, String> {
    let name = typed(form, "name");
    free_name(reading, &name)?;
    Ok(AccountChange::CreateGroup {
        name,
        members: chosen(form, "members"),
    })
}

pub fn update_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let item = group(reading, row, Changing::Update)?;
    let name = text(item, "name").unwrap_or("?");
    let gid = item.get("gid").and_then(Value::as_u64);

    let mut by_their_own: Vec<&str> = Vec::new();
    let mut choices: Vec<Choice> = Vec::new();
    for (account, primary) in accounts(reading) {
        if gid.is_some() && primary == gid {
            by_their_own.push(account);
            continue;
        }
        choices.push(Choice::of(
            account,
            members(item).any(|member| member == account),
        ));
    }
    let listed = Field::choices("members", "members", choices);
    let listed = match by_their_own.is_empty() {
        true => listed,
        false => listed.hinted(format!(
            "{} in it by their own account's group, and that is not changed here",
            by_their_own.join(", ")
        )),
    };

    Ok(Form::new(format!("EDIT THE GROUP {name}"))
        .saying(
            "The agent runs groupmod -n to rename it and gpasswd -M to set its members, for \
             what is changed below.",
        )
        .with(Field::fixed("name", "name", name))
        .with(Field::text("rename", "new name", "").hinted("left empty, the name stays as it is"))
        .with(listed))
}

pub fn update(
    reading: &Snapshot,
    row: Option<&RowKey>,
    form: &Form,
) -> Result<AccountChange, String> {
    let item = group(reading, row, Changing::Update)?;
    let rename = changed_text(form, "rename");
    if let Some(rename) = &rename {
        free_name(reading, rename)?;
    }
    let members = match form.changed("members") {
        true => Some(chosen(form, "members")),
        false => None,
    };
    Ok(AccountChange::UpdateGroup {
        name: text(item, "name").unwrap_or_default().to_string(),
        rename,
        members,
    })
}

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    let item = group(reading, row, Changing::Delete)?;
    if item.get("gid").and_then(Value::as_u64) == Some(0) {
        return Err("gid 0 is the group root: this console does not delete it".to_string());
    }
    Ok(AccountChange::DeleteGroup {
        name: text(item, "name").unwrap_or_default().to_string(),
    })
}
