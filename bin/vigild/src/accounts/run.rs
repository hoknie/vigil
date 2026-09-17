use vigil_model::{AccountChange, ChangeReport, Changed, Rfc3339, Snapshot};
use vigil_users::parse_authorized_keys;

use super::aims::aim;
use super::carried::Carried;
use super::carry::carry;
use super::ours::Ours;
use super::plans::plan;
use crate::files;

pub const READING: &str = "users";

pub const MOST_AT_ONCE: usize = 64;

const NOT_READ: &str = "the agent has not read the accounts of this host yet";

pub fn carry_out(changes: &[AccountChange], reading: Option<&Snapshot>, now: Rfc3339) -> Carried {
    let ours = Ours::here();
    asked(changes, now, &mut |change| one(change, reading, ours))
}

fn asked(
    changes: &[AccountChange],
    now: Rfc3339,
    each: &mut dyn FnMut(&AccountChange) -> Changed,
) -> Carried {
    let mut changed: Vec<Changed> = Vec::with_capacity(changes.len());
    let mut asked: Vec<String> = Vec::with_capacity(changes.len());
    let mut seen: Vec<&AccountChange> = Vec::with_capacity(changes.len());

    for change in changes.iter().take(MOST_AT_ONCE) {
        if seen.contains(&change) {
            continue;
        }
        seen.push(change);
        changed.push(each(change));
        asked.push(change.said());
    }

    for change in changes.iter().skip(MOST_AT_ONCE) {
        changed.push(Changed::refused(
            change,
            format!(
                "more than {MOST_AT_ONCE} changes were asked for at once: this one was not \
                 reached. Mark fewer and ask again."
            ),
        ));
        asked.push(change.said());
    }

    Carried {
        report: ChangeReport {
            acted_at: now,
            changed,
        },
        asked,
    }
}

fn one(change: &AccountChange, reading: Option<&Snapshot>, ours: Ours) -> Changed {
    let Some(reading) = reading else {
        return Changed::refused(change, NOT_READ);
    };
    if let Err(why) = aim(change, reading, ours) {
        return Changed::refused(change, why);
    }
    let steps = match plan(change, reading, &files::read) {
        Ok(steps) => steps,
        Err(why) => return Changed::refused(change, why),
    };

    let changed = match carry(&steps) {
        Ok(said) => Changed::done(change, said),
        Err(said) => Changed::refused(change, said),
    };
    match change {
        AccountChange::CreateKey { user, line } => match parse_authorized_keys(line).first() {
            Some(key) => changed.keyed(format!("sshkey|{user}|{}", key.fingerprint)),
            None => changed,
        },
        _ => changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-14T10:00:00.000Z".into()
    }

    fn delete(name: &str) -> AccountChange {
        AccountChange::DeleteUser { name: name.into() }
    }

    #[test]
    fn a_change_named_twice_in_one_ask_is_made_once() {
        let mut made = 0;

        let carried = asked(&[delete("eve"), delete("eve")], now(), &mut |change| {
            made += 1;
            Changed::done(change, "done")
        });

        assert_eq!(
            made, 1,
            "userdel twice is a second refusal nobody asked to read"
        );
        assert_eq!(carried.report.changed.len(), 1);
        assert_eq!(carried.asked.len(), carried.report.changed.len());
    }

    #[test]
    fn an_ask_longer_than_the_ceiling_is_cut_and_every_change_past_it_says_so() {
        let many: Vec<AccountChange> = (0..MOST_AT_ONCE + 2)
            .map(|at| delete(&format!("user{at}")))
            .collect();

        let carried = asked(&many, now(), &mut |change| Changed::done(change, "done"));

        assert_eq!(carried.report.changed.len(), many.len());
        assert_eq!(carried.report.done(), MOST_AT_ONCE);
        assert!(
            carried.report.changed[MOST_AT_ONCE]
                .said
                .contains("Mark fewer"),
            "a change silently dropped is a change the operator believes was made"
        );
        assert_eq!(carried.asked.len(), carried.report.changed.len());
    }

    #[test]
    fn with_no_reading_behind_it_the_agent_changes_nothing_and_says_why_on_every_row() {
        let carried = carry_out(&[delete("contractor")], None, now());

        assert_eq!(carried.report.done(), 0);
        assert!(carried.report.changed[0].said.contains("has not read"));
    }

    #[test]
    fn the_accounts_are_looked_up_in_the_reading_of_users() {
        assert_eq!(READING, "users");
    }
}
