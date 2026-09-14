use serde_json::Value;
use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::{Field, Form, RowKey};

use crate::helpers::{NOTHING_CHANGED, expect_kind, row_of, sudo_rule, typed};
use crate::types::Kind;
use crate::views::fields::{flag, rules, text};

const DIRECTORY: &str = "/etc/sudoers.d/";

const EDITABLE: &[&str] = &[
    "rule_1", "rule_2", "rule_3", "rule_4", "rule_5", "rule_6", "rule_7", "rule_8", "rule_9",
    "rule_10", "rule_11", "rule_12", "rule_13", "rule_14", "rule_15", "rule_16",
];

const KEPT: &[&str] = &[
    "kept_1", "kept_2", "kept_3", "kept_4", "kept_5", "kept_6", "kept_7", "kept_8", "kept_9",
    "kept_10", "kept_11", "kept_12", "kept_13", "kept_14", "kept_15", "kept_16",
];

struct Rule<'a> {
    source: &'a str,
    spec: &'a str,
}

impl Rule<'_> {
    fn editable(&self) -> bool {
        self.source.starts_with(DIRECTORY)
    }
}

fn grant<'a>(
    reading: &'a Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<(String, Vec<Rule<'a>>), String> {
    let (key, item) = row_of(reading, row, changing)?;
    expect_kind(key, Kind::Sudoer, "a sudo grant")?;
    let who = text(item, "who").unwrap_or_default().to_string();
    let found: Vec<Rule<'a>> = rules(item).map(rule).collect();

    if !found.iter().any(Rule::editable) {
        return Err(format!(
            "every rule of {who} lives in /etc/sudoers, which this console does not change: a \
             mistake there costs this host sudo. Edit it with visudo."
        ));
    }
    Ok((who, found))
}

fn rule(value: &Value) -> Rule<'_> {
    Rule {
        source: text(value, "source").unwrap_or_default(),
        spec: text(value, "spec").unwrap_or_default(),
    }
}

fn hidden(reading: &Snapshot, row: Option<&RowKey>) -> bool {
    row.and_then(|row| reading.items.get(&row.key))
        .is_some_and(|item| {
            flag(item, "spec_redacted") || rules(item).any(|one| flag(one, "spec_redacted"))
        })
}

pub fn update_form(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let (who, found) = grant(reading, row, Changing::Update)?;
    if hidden(reading, row) {
        return Err(format!(
            "the reading hides part of the grant of {who}: a secret was taken out of it, and \
             writing it back from here would write the marks instead. Edit the file by hand."
        ));
    }
    let editable = found.iter().filter(|one| one.editable()).count();
    if editable > EDITABLE.len() || found.len() - editable > KEPT.len() {
        return Err(format!(
            "{who} has more rules than this form holds ({}): edit the file by hand",
            EDITABLE.len()
        ));
    }

    let mut form = Form::new(format!("EDIT THE SUDO GRANT OF {who}"))
        .saying(format!(
            "The agent writes the rules of {who} into its file under {DIRECTORY}, checks the \
             result with visudo -cf, and puts it in place only if the check passes."
        ))
        .with(Field::fixed("who", "who", who.as_str()));
    let (mut edited, mut kept) = (0, 0);
    for (at, one) in found.iter().enumerate() {
        let label = format!("rule {}", at + 1);
        form = match one.editable() {
            true => {
                edited += 1;
                form.with(
                    Field::text(EDITABLE[edited - 1], label, one.spec)
                        .hinted(format!("in {}; emptied, the rule goes", one.source)),
                )
            }
            false => {
                kept += 1;
                form.with(
                    Field::fixed(KEPT[kept - 1], label, one.spec).hinted(format!(
                        "lives in {} and is not changed from here",
                        one.source
                    )),
                )
            }
        };
    }
    Ok(form.with(
        Field::text("new_rule", "new rule", "")
            .hinted("as in ALL=(ALL) ALL: every command as anyone, after a password"),
    ))
}

pub fn update(
    reading: &Snapshot,
    row: Option<&RowKey>,
    form: &Form,
) -> Result<AccountChange, String> {
    let (who, _) = grant(reading, row, Changing::Update)?;
    if !form.fields.iter().any(Field::changed) {
        return Err(NOTHING_CHANGED.to_string());
    }

    let mut rules: Vec<String> = Vec::new();
    for name in EDITABLE.iter().chain(["new_rule"].iter()) {
        if form.field(name).is_none() {
            continue;
        }
        let rule = typed(form, name);
        if rule.is_empty() {
            continue;
        }
        sudo_rule(&rule)?;
        rules.push(rule);
    }
    if rules.is_empty() {
        return Err(format!(
            "every rule of {who} was emptied: to take the grant away, leave this form and press D"
        ));
    }
    Ok(AccountChange::UpdateSudo { who, rules })
}

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    let (who, _) = grant(reading, row, Changing::Delete)?;
    Ok(AccountChange::DeleteSudo { who })
}
