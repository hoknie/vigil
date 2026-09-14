use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Choice, Field, Form, RowKey};

use super::key::unreadable;
use crate::helpers::{LINE_HINT, key_line, name_accepted, row_of, typed, unchosen};
use crate::types::Kind;
use crate::views::facts::{keys_of, readable};
use crate::views::fields::text;

fn user_of(reading: &Snapshot, row: Option<&RowKey>, changing: Changing) -> Result<String, String> {
    let (key, item) = row_of(reading, row, changing)?;
    let user = match Kind::of(key) {
        Kind::Account => text(item, "name"),
        Kind::Key => text(item, "user"),
        _ => None,
    };
    match user {
        Some(user) => Ok(user.to_string()),
        None => Err(format!("{key} is not an account let in by a key")),
    }
}

pub fn create_form() -> Form {
    Form::new("NEW SSH USER")
        .saying(
            "The agent writes ~/.ssh/authorized_keys for the account with this one key: owned \
             by the account, mode 0600, in a .ssh of mode 0700.",
        )
        .with(
            Field::text("account", "account", "")
                .hinted("an account of this host that no key lets in yet"),
        )
        .with(Field::text("line", "key line", "").hinted(LINE_HINT))
}

pub fn create(reading: &Snapshot, form: &Form) -> Result<AccountChange, String> {
    let user = typed(form, "account");
    if user.is_empty() {
        return Err("which account is let in? Type its name into the account field".to_string());
    }
    name_accepted("the account", &user)?;
    if !reading.items.contains_key(&format!("account|{user}")) {
        return Err(format!("there is no account {user} in the reading"));
    }
    if keys_of(reading, &user).next().is_some() {
        return Err(format!(
            "{user} is let in by a key already: add another from the keys list, or edit this one"
        ));
    }
    let line = typed(form, "line");
    key_line(&line)?;
    Ok(AccountChange::CreateSshUser { user, line })
}

pub fn update_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let user = user_of(reading, row, Changing::Update)?;
    let mut choices: Vec<Choice> = Vec::new();
    for (_, item) in keys_of(reading, &user) {
        if !readable(item) {
            return Err(unreadable(&user));
        }
        let fingerprint = text(item, "fingerprint").unwrap_or("?");
        choices.push(Choice::of(
            match text(item, "comment") {
                Some(comment) if !comment.is_empty() => format!("{fingerprint} {comment}"),
                _ => fingerprint.to_string(),
            },
            true,
        ));
    }

    Ok(Form::new(format!("EDIT THE KEYS OF {user}"))
        .saying(format!(
            "The agent rewrites ~/.ssh/authorized_keys of {user}: a key left unchosen goes, and \
             a line typed below is added."
        ))
        .with(Field::fixed("account", "account", user.as_str()))
        .with(Field::choices("keys", "keys", choices).hinted("a key left unchosen is taken away"))
        .with(Field::text("add_key", "add a key", "").hinted(LINE_HINT)))
}

pub fn update(
    reading: &Snapshot,
    row: Option<&RowKey>,
    form: &Form,
) -> Result<AccountChange, String> {
    let user = user_of(reading, row, Changing::Update)?;
    let removed: Vec<String> = unchosen(form, "keys")
        .into_iter()
        .filter_map(|named| named.split_whitespace().next().map(str::to_string))
        .collect();

    let mut added: Vec<String> = Vec::new();
    let line = typed(form, "add_key");
    if !line.is_empty() {
        let parsed = key_line(&line)?;
        if reading
            .items
            .contains_key(&format!("sshkey|{user}|{}", parsed.fingerprint))
        {
            return Err(format!("that key already lets {user} in"));
        }
        added.push(line);
    }

    Ok(AccountChange::UpdateSshUser {
        user,
        removed,
        added,
    })
}

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    Ok(AccountChange::DeleteSshUser {
        user: user_of(reading, row, Changing::Delete)?,
    })
}
