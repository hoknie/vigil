use std::fs;
use std::io::{self, ErrorKind};

use vigil_collect::{Health, Limit, Standing, denied_to, this_account};

use super::directory::accounts;
use super::logins::read_logins;
use super::sudoers::SUDOERS;

pub(super) fn standing() -> Standing {
    if accounts().is_empty() {
        return Standing::of(Health::Unavailable(
            "Directory Services answered with no account at all, so who may log in to this \
             host is unknown"
                .into(),
        ));
    }

    let sudoers = fs::read_to_string(SUDOERS).map(|_| ());
    let (_, source) = read_logins();
    let logins = source.reason.filter(|_| !source.read);

    judged(sudoers, logins, this_account())
}

pub(super) fn judged(sudoers: io::Result<()>, logins: Option<String>, uid: u32) -> Standing {
    let mut missing: Vec<String> = Vec::new();
    let mut unseen: Vec<String> = Vec::new();

    match sudoers {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) if denied_to(error.kind(), uid) => unseen.push(format!(
            "{SUDOERS} is not readable: grants made there and in the files beside it will not \
             be seen; run as root"
        )),
        Err(error) => missing.push(format!(
            "{SUDOERS} could not be read ({error}): grants made there will not be seen"
        )),
    }
    if let Some(reason) = logins {
        missing.push(format!("{reason}: who is logged in will be missing"));
    }

    let health = match missing.is_empty() {
        true => Health::Ok,
        false => Health::Degraded(missing.join("; ")),
    };
    Standing::limited(health, Limit::of(unseen))
}
