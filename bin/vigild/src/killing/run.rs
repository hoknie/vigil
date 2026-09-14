use vigil_model::{KillReport, Killed, Killing, Rfc3339, Snapshot};

use super::destruction::destroy;
use super::report::killed;
use super::signal::send;
use super::targets::{Aim, aim};

pub const MOST_AT_ONCE: usize = 64;

pub fn carry_out(
    sockets: &[String],
    killing: Killing,
    reading: Option<&Snapshot>,
    now: Rfc3339,
) -> KillReport {
    let ours = std::process::id();
    let mut done: Vec<Killed> = Vec::with_capacity(sockets.len());
    let mut seen: Vec<&String> = Vec::with_capacity(sockets.len());

    for key in sockets.iter().take(MOST_AT_ONCE) {
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);

        done.push(match aim(key, reading, killing, ours) {
            Aim::Nowhere(refused) => refused,
            Aim::At(target) => {
                let outcome = match killing {
                    Killing::Destroy => destroy(&target),
                    signalled => send(&target, signalled),
                };
                match outcome {
                    Ok(said) => killed(&target, &said),
                    Err(said) => Killed::refused(target.key.clone(), said)
                        .about(Some(target.pid), target.program.clone()),
                }
            }
        });
    }

    for key in sockets.iter().skip(MOST_AT_ONCE) {
        done.push(Killed::refused(
            key.clone(),
            format!(
                "more than {MOST_AT_ONCE} socket(s) were asked for at once: this one was \
                     not reached. Mark fewer and ask again."
            ),
        ));
    }

    KillReport {
        killing,
        acted_at: now,
        killed: done,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_socket_named_twice_in_one_ask_is_acted_on_once() {
        let report = carry_out(
            &["tcp|0.0.0.0:4444".into(), "tcp|0.0.0.0:4444".into()],
            Killing::Terminate,
            None,
            "2026-09-14T10:00:00.000Z".into(),
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
        let many: Vec<String> = (0..MOST_AT_ONCE + 3)
            .map(|port| format!("tcp|0.0.0.0:{}", 9000 + port))
            .collect();

        let report = carry_out(
            &many,
            Killing::Terminate,
            None,
            "2026-09-14T10:00:00.000Z".into(),
        );

        assert_eq!(report.killed.len(), many.len());
        assert_eq!(report.done(), 0);
        assert!(
            report.killed[MOST_AT_ONCE].said.contains("Mark fewer"),
            "a row silently dropped from a kill is a row the operator believes is dead: {}",
            report.killed[MOST_AT_ONCE].said
        );
    }

    #[test]
    fn with_no_reading_behind_it_the_agent_touches_nothing_and_says_why_on_every_row() {
        let report = carry_out(
            &["tcp|0.0.0.0:4444".into()],
            Killing::Kill,
            None,
            "2026-09-14T10:00:00.000Z".into(),
        );

        assert_eq!(report.done(), 0);
        assert!(
            report.killed[0].said.contains("has not read"),
            "{:?}",
            report.killed[0]
        );
    }
}
