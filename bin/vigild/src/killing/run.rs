use vigil_model::{KillReport, KillTarget, Killed, Killing, Rfc3339, Snapshot};

use super::destruction::destroy;
use super::programs;
use super::report::killed;
use super::signal::send;
use super::targets::{self, Aim, Target};

pub const MOST_AT_ONCE: usize = 64;

pub fn reading_of(target: KillTarget) -> &'static str {
    match target {
        KillTarget::Socket => targets::READING,
        KillTarget::Program => programs::READING,
    }
}

pub fn carry_out(
    target: KillTarget,
    keys: &[String],
    killing: Killing,
    reading: Option<&Snapshot>,
    now: Rfc3339,
) -> KillReport {
    let ours = std::process::id();

    asked(target, keys, killing, now, &mut |key| match target {
        KillTarget::Socket => socket(key, killing, reading, ours),
        KillTarget::Program => program(key, killing, reading, ours),
    })
}

fn asked(
    target: KillTarget,
    keys: &[String],
    killing: Killing,
    now: Rfc3339,
    each: &mut dyn FnMut(&str) -> Killed,
) -> KillReport {
    let mut done: Vec<Killed> = Vec::with_capacity(keys.len());
    let mut seen: Vec<&String> = Vec::with_capacity(keys.len());

    for key in keys.iter().take(MOST_AT_ONCE) {
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        done.push(each(key));
    }

    for key in keys.iter().skip(MOST_AT_ONCE) {
        done.push(Killed::refused(
            key.clone(),
            format!(
                "more than {MOST_AT_ONCE} {}(s) were asked for at once: this one was not \
                 reached. Mark fewer and ask again.",
                target.as_str()
            ),
        ));
    }

    KillReport {
        target,
        killing,
        acted_at: now,
        killed: done,
    }
}

fn socket(key: &str, killing: Killing, reading: Option<&Snapshot>, ours: u32) -> Killed {
    match targets::aim(key, reading, killing, ours) {
        Aim::Nowhere(refused) => refused,
        Aim::At(target) => {
            let outcome = match killing {
                Killing::Destroy => destroy(&target),
                signalled => send(target.pid, signalled, &|pid| still_holds(&target, pid)),
            };
            match outcome {
                Ok(said) => killed(&target, &said),
                Err(said) => Killed::refused(target.key.clone(), said)
                    .about(Some(target.pid), target.program.clone()),
            }
        }
    }
}

fn still_holds(target: &Target, pid: u32) -> bool {
    target
        .program
        .as_deref()
        .is_none_or(|executable| vigil_processes::still_running(pid, executable, None))
}

fn program(key: &str, killing: Killing, reading: Option<&Snapshot>, ours: u32) -> Killed {
    match programs::aim(key, reading, killing, ours, &vigil_processes::running) {
        programs::Aim::Nowhere(refused) => refused,
        programs::Aim::At(program) => programs::stop(&program, killing, &|pid, killing| {
            send(pid, killing, &|pid| {
                vigil_processes::still_running(pid, &program.executable, Some(program.uid))
            })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-14T10:00:00.000Z".into()
    }

    #[test]
    fn a_socket_named_twice_in_one_ask_is_acted_on_once() {
        let report = carry_out(
            KillTarget::Socket,
            &["tcp|0.0.0.0:4444".into(), "tcp|0.0.0.0:4444".into()],
            Killing::Terminate,
            None,
            now(),
        );

        assert_eq!(
            report.killed.len(),
            1,
            "the same row can be marked in both lists of the section, and signalling a \
             process twice because a key arrived twice is a SIGKILL nobody asked for"
        );
    }

    #[test]
    fn an_ask_longer_than_the_ceiling_is_cut_and_every_row_past_it_says_so() {
        for target in [KillTarget::Socket, KillTarget::Program] {
            let many: Vec<String> = (0..MOST_AT_ONCE + 3)
                .map(|port| format!("tcp|0.0.0.0:{}", 9000 + port))
                .collect();

            let report = carry_out(target, &many, Killing::Terminate, None, now());

            assert_eq!(report.killed.len(), many.len());
            assert_eq!(report.done(), 0);
            assert!(
                report.killed[MOST_AT_ONCE].said.contains("Mark fewer")
                    && report.killed[MOST_AT_ONCE].said.contains(target.as_str()),
                "a row silently dropped from a kill is a row the operator believes is dead: {}",
                report.killed[MOST_AT_ONCE].said
            );
        }
    }

    #[test]
    fn with_no_reading_behind_it_the_agent_touches_nothing_and_says_why_on_every_row() {
        for (target, key) in [
            (KillTarget::Socket, "tcp|0.0.0.0:4444"),
            (KillTarget::Program, "exec|/tmp/.x/nc|www-data"),
        ] {
            let report = carry_out(target, &[key.into()], Killing::Kill, None, now());

            assert_eq!(report.done(), 0);
            assert_eq!(report.target, target);
            assert!(
                report.killed[0].said.contains("has not read"),
                "{:?}",
                report.killed[0]
            );
        }
    }

    #[test]
    fn each_kind_of_row_is_looked_up_in_the_reading_that_holds_it() {
        assert_eq!(reading_of(KillTarget::Socket), "ports");
        assert_eq!(reading_of(KillTarget::Program), "processes");
    }
}
