use serde_json::Value;
use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Field, Form, RowKey};

use crate::helpers::{
    LINE_HINT, changed_text, expect_kind, key_line, name_accepted, one_line, row_of, typed,
};
use crate::types::Kind;
use crate::views::facts::readable;
use crate::views::fields::text;

pub fn unreadable(user: &str) -> String {
    format!(
        "the key file of {user} could not be read by the agent, so nothing in it is changed \
         from here: what the agent did not see, it would overwrite"
    )
}

fn key<'a>(
    reading: &'a Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<&'a Value, String> {
    let (found, item) = row_of(reading, row, changing)?;
    expect_kind(found, Kind::Key, "a key")?;
    if !readable(item) {
        return Err(unreadable(text(item, "user").unwrap_or("?")));
    }
    Ok(item)
}

pub fn create_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let user = match row {
        Some(under) if Kind::of(&under.key) == Kind::Key => {
            text(key(reading, row, Changing::Create)?, "user").unwrap_or_default()
        }
        _ => "",
    };
    Ok(Form::new("NEW KEY")
        .saying(
            "The agent adds the line to the account's ~/.ssh/authorized_keys, keeping the \
             file's owner and mode.",
        )
        .with(Field::text("account", "account", user))
        .with(Field::text("line", "key line", "").hinted(LINE_HINT)))
}

pub fn create(reading: &Snapshot, form: &Form) -> Result<AccountChange, String> {
    let user = typed(form, "account");
    if user.is_empty() {
        return Err(
            "which account does the key let in? Type its name into the account field".to_string(),
        );
    }
    name_accepted("the account", &user)?;
    if !reading.items.contains_key(&format!("account|{user}")) {
        return Err(format!("there is no account {user} in the reading"));
    }
    if reading
        .items
        .contains_key(&format!("sshkey|{user}|unreadable"))
    {
        return Err(unreadable(&user));
    }
    let line = typed(form, "line");
    let parsed = key_line(&line)?;
    if reading
        .items
        .contains_key(&format!("sshkey|{user}|{}", parsed.fingerprint))
    {
        return Err(format!("that key already lets {user} in"));
    }
    Ok(AccountChange::CreateKey { user, line })
}

pub fn update_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let item = key(reading, row, Changing::Update)?;
    let user = text(item, "user").unwrap_or("?");
    let fingerprint = text(item, "fingerprint").unwrap_or("?");
    Ok(Form::new(format!("EDIT THE KEY {fingerprint}"))
        .saying(format!(
            "The agent rewrites this key's line in {} with the options and the comment below; \
             the key itself stays.",
            text(item, "source").unwrap_or("authorized_keys")
        ))
        .with(Field::fixed("account", "account", user))
        .with(Field::fixed(
            "algorithm",
            "algorithm",
            text(item, "algorithm").unwrap_or("?"),
        ))
        .with(Field::fixed("fingerprint", "fingerprint", fingerprint))
        .with(
            Field::text(
                "options",
                "options",
                text(item, "options").unwrap_or_default(),
            )
            .hinted("as in from=\"10.0.0.0/8\",no-pty; emptied, the key has none"),
        )
        .with(Field::text(
            "comment",
            "comment",
            text(item, "comment").unwrap_or_default(),
        )))
}

pub fn update(
    reading: &Snapshot,
    row: Option<&RowKey>,
    form: &Form,
) -> Result<AccountChange, String> {
    let item = key(reading, row, Changing::Update)?;
    let options = changed_text(form, "options");
    if let Some(options) = &options {
        one_line("the options", options)?;
    }
    let comment = changed_text(form, "comment");
    if let Some(comment) = &comment {
        one_line("the comment", comment)?;
    }
    Ok(AccountChange::UpdateKey {
        user: text(item, "user").unwrap_or_default().to_string(),
        fingerprint: text(item, "fingerprint").unwrap_or_default().to_string(),
        options,
        comment,
    })
}

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    let item = key(reading, row, Changing::Delete)?;
    Ok(AccountChange::DeleteKey {
        user: text(item, "user").unwrap_or_default().to_string(),
        fingerprint: text(item, "fingerprint").unwrap_or_default().to_string(),
    })
}
