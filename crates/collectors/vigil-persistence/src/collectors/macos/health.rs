use vigil_collect::Health;

use super::cron::read_cron;
use super::hooks::read_hooks;
use super::jobs::{homes, unseen};

pub(super) fn health() -> Health {
    let homes = homes();
    let (jobs, unlisted, unreadable) = unseen(&homes);
    let mut missing: Vec<String> = Vec::new();

    if !unlisted.is_empty() {
        missing.push(format!(
            "the launchd jobs in {} cannot be listed, so what starts from them is not seen",
            unlisted.join(", ")
        ));
    }
    if !unreadable.is_empty() {
        missing.push(format!(
            "{} launchd job file(s) cannot be read, so what they start is unknown: {}",
            unreadable.len(),
            unreadable.join(", ")
        ));
    }
    if let (_, Some(why)) = read_cron() {
        missing.push(format!(
            "the crontab of every account is not read ({why}); the directory is root's alone"
        ));
    }
    if let Err(why) = read_hooks() {
        missing.push(format!(
            "the login and logout hooks of this Mac are not read ({why}); root's preferences \
             are root's alone"
        ));
    }

    match (jobs == 0, missing.is_empty()) {
        (true, _) => Health::Unavailable(format!(
            "not one launchd job could be read, so what this Mac starts by itself is unknown: {}",
            missing.join("; ")
        )),
        (false, true) => Health::Ok,
        (false, false) => Health::Degraded(format!("{}; run as root", missing.join("; "))),
    }
}
