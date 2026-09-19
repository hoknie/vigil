use vigil_collect::{CollectError, Health};

use super::collector::{EnginesCollector, Held};
use crate::types::WRITER;

const JOB: &str = "vigil.containers";

const LOG: &str = "/usr/local/var/log/vigil/vigil-containers.log";

fn how_it_is_written() -> String {
    format!(
        "this reading is written by the launchd job {JOB}, which runs {WRITER} and nothing \
         else; the agent never starts a program of its own"
    )
}

pub enum Standing {
    Read,
    NotOne(CollectError),
}

impl Standing {
    pub fn of(held: &[Held]) -> Standing {
        let unread: Vec<&Held> = held.iter().filter(|one| one.dump.is_err()).collect();

        match !held.is_empty() && unread.len() == held.len() {
            true => Standing::NotOne(named(&unread)),
            false => Standing::Read,
        }
    }
}

pub fn standing(collector: &EnginesCollector, held: &[Held]) -> Health {
    if held.is_empty() {
        return Health::Unavailable(
            "no engine is named in `engines:` in the configuration file, so this reading has \
             nothing to read"
                .to_string(),
        );
    }

    if let Standing::NotOne(refusal) = Standing::of(held) {
        return refused(collector, &refusal);
    }

    let mut said: Vec<String> = Vec::new();
    for one in held {
        said.extend(complaint(collector, one));
    }

    match said.is_empty() {
        true => Health::Ok,
        false => Health::Degraded(said.join(" · ")),
    }
}

fn complaint(collector: &EnginesCollector, one: &Held) -> Vec<String> {
    let engine = one.engine.name();
    let shown = collector.dump_path(one.engine).display().to_string();
    let mut said = Vec::new();

    match &one.dump {
        Err(refusal) => said.push(format!(
            "{engine}: {refusal}, so what this Mac's {engine} holds is unknown — which is \
             not the same as an engine holding nothing"
        )),
        Ok(dump) if !dump.on_this_host() => {}
        Ok(dump) => {
            let unanswered = dump.unanswered();
            if !unanswered.is_empty() {
                said.push(format!(
                    "{engine} did not answer for {}. {}; {LOG} says what it got back. On a \
                     Mac the engine runs inside a virtual machine, and an engine whose \
                     machine is stopped answers nothing",
                    unanswered.join(", "),
                    how_it_is_written()
                ));
            }
        }
    }

    if let Some(age) = one.age_seconds {
        said.push(format!(
            "{shown} was written {age} seconds ago, more than the {} this collector allows: \
             what it says about this Mac may have been true and no longer is. {}; a job \
             launchd is not starting is the usual cause (launchctl print system/{JOB})",
            collector.stale_after_seconds(),
            how_it_is_written()
        ));
    }

    if let Some(refusal) = &one.registries_refusal {
        said.push(format!(
            "{engine}: {refusal}, so which registries this Mac may pull from is unknown"
        ));
    }

    said
}

fn refused(collector: &EnginesCollector, refusal: &CollectError) -> Health {
    let where_they_live = collector.dump_path(crate::types::Engine::Docker);
    let directory = where_they_live
        .parent()
        .map(|at| at.display().to_string())
        .unwrap_or_default();

    match refusal {
        CollectError::Absent(_) => Health::Unavailable(format!(
            "nothing is written in {directory}, so what this Mac's container engines hold is \
             unknown — which is not the same as a Mac with no containers on it. {}. Either \
             the job has never run (vigild collector containers-engines enable), or it was \
             switched off",
            how_it_is_written()
        )),
        CollectError::Denied(_) => Health::Unavailable(format!(
            "{directory} cannot be read by this agent, so what this Mac's container engines \
             hold is unknown. The directory is 0700 root:wheel and so are the files in it; \
             run as root"
        )),
        other => Health::Degraded(format!(
            "{other}. {}; its last run left this behind. {LOG} and `launchctl print \
             system/{JOB}` say why",
            how_it_is_written()
        )),
    }
}

fn named(unread: &[&Held]) -> CollectError {
    let mut absent = 0;
    let mut denied = 0;
    let mut said: Vec<String> = Vec::new();

    for one in unread {
        match &one.dump {
            Err(CollectError::Absent(what)) => {
                absent += 1;
                said.push(what.clone());
            }
            Err(CollectError::Denied(what)) => {
                denied += 1;
                said.push(what.clone());
            }
            Err(other) => said.push(other.to_string()),
            Ok(_) => continue,
        }
    }

    let together = said.join(", ");
    match (denied > 0, absent == unread.len()) {
        (true, _) => CollectError::Denied(together),
        (false, true) => CollectError::Absent(together),
        (false, false) => CollectError::Unreadable(together),
    }
}
