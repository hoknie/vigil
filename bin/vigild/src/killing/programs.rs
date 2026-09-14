use vigil_model::{Killed, Killing, Snapshot};
use vigil_processes::ProcessView;

use super::signal::named;

pub const READING: &str = "processes";

const PIDS_WRITTEN_OUT: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub key: String,
    pub executable: String,
    pub uid: u32,
    pub pids: Vec<u32>,
}

pub enum Aim {
    At(Program),
    Nowhere(Killed),
}

pub type Running<'a> = &'a dyn Fn(&str, u32) -> Result<Vec<u32>, String>;

pub type Signalling<'a> = &'a dyn Fn(u32, Killing) -> Result<String, String>;

pub fn aim(
    key: &str,
    reading: Option<&Snapshot>,
    killing: Killing,
    ours: u32,
    running: Running<'_>,
) -> Aim {
    let Some(reading) = reading else {
        return Aim::Nowhere(Killed::refused(
            key,
            "the agent has not read the programs running on this host yet",
        ));
    };
    let Some(item) = reading.items.get(key) else {
        return Aim::Nowhere(Killed::refused(
            key,
            "no longer in the reading the agent holds: nothing runs it any more, or it was \
             never there",
        ));
    };

    let view = ProcessView::new(item);
    if !view.is_program() {
        return Aim::Nowhere(Killed::refused(
            key,
            "this row is about the reading itself, it is not a program",
        ));
    }
    let executable = view.executable().to_string();
    if !killing.touches_the_process() {
        return Aim::Nowhere(
            Killed::refused(
                key,
                "a program has no socket of its own to close: stopping its processes is what \
                 this row offers",
            )
            .about(None, Some(executable)),
        );
    }
    let Ok(uid) = u32::try_from(view.uid()) else {
        return Aim::Nowhere(
            Killed::refused(key, "the reading carries no account for this program")
                .about(None, Some(executable)),
        );
    };

    let pids = match running(&executable, uid) {
        Ok(pids) => pids,
        Err(said) => {
            return Aim::Nowhere(
                Killed::refused(
                    key,
                    format!("the processes running it could not be found: {said}"),
                )
                .about(None, Some(executable)),
            );
        }
    };
    if pids.is_empty() {
        return Aim::Nowhere(
            Killed::refused(key, format!("nothing runs it as {} any more", view.user()))
                .about(None, Some(executable)),
        );
    }
    if pids.contains(&1) {
        return Aim::Nowhere(
            Killed::refused(
                key,
                "pid 1 runs this program: stopping it stops the host, and this agent does \
                 not do that",
            )
            .about(Some(1), Some(executable)),
        );
    }
    if pids.contains(&ours) {
        return Aim::Nowhere(
            Killed::refused(
                key,
                "this agent is one of the processes running it: it does not kill itself on \
                 request",
            )
            .about(Some(ours), Some(executable)),
        );
    }

    Aim::At(Program {
        key: key.to_string(),
        executable,
        uid,
        pids,
    })
}

pub fn stop(program: &Program, killing: Killing, signalling: Signalling<'_>) -> Killed {
    let mut sent: Vec<u32> = Vec::with_capacity(program.pids.len());
    let mut left: Vec<String> = Vec::new();
    for pid in &program.pids {
        match signalling(*pid, killing) {
            Ok(_) => sent.push(*pid),
            Err(said) => left.push(said),
        }
    }

    let executable = Some(program.executable.clone());
    let Some(lowest) = sent.first().copied() else {
        return Killed::refused(program.key.clone(), left.join("; "))
            .about(program.pids.first().copied(), executable);
    };

    let mut said = format!(
        "{} sent to {} process(es): {}",
        named(killing),
        sent.len(),
        written_out(&sent)
    );
    if !left.is_empty() {
        said.push_str(&format!(
            "; {} not signalled: {}",
            left.len(),
            left.join("; ")
        ));
    }
    Killed::done(program.key.clone(), lowest, executable, &said)
}

pub fn written_out(pids: &[u32]) -> String {
    let named: Vec<String> = pids
        .iter()
        .take(PIDS_WRITTEN_OUT)
        .map(u32::to_string)
        .collect();
    match pids.len() > PIDS_WRITTEN_OUT {
        true => format!(
            "{} and {} more",
            named.join(", "),
            pids.len() - PIDS_WRITTEN_OUT
        ),
        false => named.join(", "),
    }
}
