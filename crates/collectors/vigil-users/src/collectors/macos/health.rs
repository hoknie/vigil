use std::fs;
use std::io::ErrorKind;

use vigil_collect::Health;

use super::directory::accounts;
use super::logins::read_logins;
use super::sudoers::SUDOERS;

pub(super) fn health() -> Health {
    if accounts().is_empty() {
        return Health::Unavailable(
            "Directory Services answered with no account at all, so who may log in to this \
             host is unknown"
                .into(),
        );
    }

    let mut missing: Vec<String> = Vec::new();
    match fs::read_to_string(SUDOERS) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) if error.kind() == ErrorKind::PermissionDenied => missing.push(format!(
            "{SUDOERS} is not readable: grants made there and in the files beside it will not \
             be seen; run as root"
        )),
        Err(error) => missing.push(format!(
            "{SUDOERS} could not be read ({error}): grants made there will not be seen"
        )),
    }
    let (_, source) = read_logins();
    if let Some(reason) = source.reason.filter(|_| !source.read) {
        missing.push(format!("{reason}: who is logged in will be missing"));
    }

    match missing.is_empty() {
        true => Health::Ok,
        false => Health::Degraded(missing.join("; ")),
    }
}
