use vigil_collect::CollectError;
use vigil_model::Snapshot;

use super::directory::{accounts, groups};
use super::keys::read_authorized_keys;
use super::logins::read_logins;
use super::sudoers::read_sudoers;
use crate::parsers::{
    AccountsReading, accounts_snapshot, merge_sessions, merged_accounts, merged_groups,
};

pub(super) fn reading(taken_at: &str) -> Result<Snapshot, CollectError> {
    let (passwd, facts) = merged_accounts(accounts());
    if passwd.is_empty() {
        return Err(CollectError::Unreadable(
            "the accounts Directory Services answers with, of which there were none".into(),
        ));
    }

    let groups = merged_groups(groups());
    let sudo = read_sudoers();
    let keys = read_authorized_keys(&passwd);
    let (logins, source) = read_logins();
    let sessions = merge_sessions(logins);

    Ok(accounts_snapshot(
        taken_at,
        &AccountsReading {
            passwd: &passwd,
            groups: &groups,
            shadow: Some(&facts),
            sudo: &sudo,
            keys: &keys,
            sessions: &sessions,
            session_sources: std::slice::from_ref(&source),
        },
    ))
}
