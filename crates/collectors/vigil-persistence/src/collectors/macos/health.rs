use vigil_collect::{Health, Limit, Standing};

use super::cron::read_cron;
use super::files::Unread;
use super::hooks::read_hooks;
use super::jobs::{homes, unseen};

pub(super) struct Unseen {
    pub(super) jobs: usize,
    pub(super) unlisted: Vec<Unread>,
    pub(super) unopened: Vec<Unread>,
    pub(super) cron: Option<Unread>,
    pub(super) hooks: Option<Unread>,
}

pub(super) fn standing() -> Standing {
    let homes = homes();
    let (jobs, unlisted, unopened) = unseen(&homes);

    judged(Unseen {
        jobs,
        unlisted,
        unopened,
        cron: read_cron().1,
        hooks: read_hooks().err(),
    })
}

pub(super) fn judged(unseen: Unseen) -> Standing {
    let mut missing: Vec<String> = Vec::new();
    let mut hidden: Vec<String> = Vec::new();

    let (refused, failed) = apart(&unseen.unlisted);
    if !refused.is_empty() {
        hidden.push(format!(
            "the launchd jobs in {} cannot be listed by this account, so what starts from them \
             is not seen",
            refused.join(", ")
        ));
    }
    if !failed.is_empty() {
        missing.push(format!(
            "the launchd jobs in {} cannot be listed, so what starts from them is not seen",
            failed.join(", ")
        ));
    }

    let (refused, failed) = apart(&unseen.unopened);
    if !refused.is_empty() {
        hidden.push(format!(
            "{} launchd job file(s) cannot be read by this account, so what they start is \
             unknown: {}",
            refused.len(),
            refused.join(", ")
        ));
    }
    if !failed.is_empty() {
        missing.push(format!(
            "{} launchd job file(s) cannot be read, so what they start is unknown: {}",
            failed.len(),
            failed.join(", ")
        ));
    }

    match &unseen.cron {
        Some(Unread::Refused(why)) => hidden.push(format!(
            "the crontab of every account is not read ({why}); the directory is root's alone"
        )),
        Some(Unread::Failed(why)) => {
            missing.push(format!("the crontab of every account is not read ({why})"))
        }
        None => {}
    }
    match &unseen.hooks {
        Some(Unread::Refused(why)) => hidden.push(format!(
            "the login and logout hooks of this Mac are not read ({why}); root's preferences \
             are root's alone"
        )),
        Some(Unread::Failed(why)) => missing.push(format!(
            "the login and logout hooks of this Mac are not read ({why})"
        )),
        None => {}
    }

    if unseen.jobs == 0 {
        let everything: Vec<String> = hidden.iter().chain(missing.iter()).cloned().collect();
        return Standing::of(Health::Unavailable(format!(
            "not one launchd job could be read, so what this Mac starts by itself is unknown: {}",
            everything.join("; ")
        )));
    }

    let health = match missing.is_empty() {
        true => Health::Ok,
        false => Health::Degraded(missing.join("; ")),
    };
    let limit = match hidden.is_empty() {
        true => None,
        false => Limit::of(vec![format!("{}; run as root", hidden.join("; "))]),
    };
    Standing::limited(health, limit)
}

fn apart(unread: &[Unread]) -> (Vec<&str>, Vec<&str>) {
    let refused = unread
        .iter()
        .filter(|one| matches!(one, Unread::Refused(_)))
        .map(Unread::said)
        .collect();
    let failed = unread
        .iter()
        .filter(|one| matches!(one, Unread::Failed(_)))
        .map(Unread::said)
        .collect();
    (refused, failed)
}
