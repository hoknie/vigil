use vigil_model::{ControlReport, ControlTarget, Controlled, Controlling, Rfc3339, Snapshot};
use vigil_persistence::CronJob;

use super::cron;
use super::names;
use super::systemctl;

pub const READING: &str = "persistence";

pub const MOST_AT_ONCE: usize = 64;

const NOT_READ: &str = "the agent has not read what this host starts by itself yet, so there is \
                        nothing to look this row up in";

pub fn carry_out(
    keys: &[String],
    controlling: Controlling,
    reading: Option<&Snapshot>,
    now: Rfc3339,
) -> ControlReport {
    asked(keys, controlling, now, &mut |key| {
        one(key, controlling, reading)
    })
}

pub fn refused(
    keys: &[String],
    controlling: Controlling,
    now: Rfc3339,
    why: &str,
) -> ControlReport {
    asked(keys, controlling, now, &mut |key| {
        Controlled::refused(key, why)
    })
}

fn asked(
    keys: &[String],
    controlling: Controlling,
    now: Rfc3339,
    each: &mut dyn FnMut(&str) -> Controlled,
) -> ControlReport {
    let mut done: Vec<Controlled> = Vec::with_capacity(keys.len());
    let mut seen: Vec<&String> = Vec::with_capacity(keys.len());

    for key in keys.iter().take(MOST_AT_ONCE) {
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        done.push(each(key));
    }

    for key in keys.iter().skip(MOST_AT_ONCE) {
        done.push(Controlled::refused(
            key.clone(),
            format!(
                "more than {MOST_AT_ONCE} rows were asked for at once: this one was not \
                 reached. Mark fewer and ask again."
            ),
        ));
    }

    ControlReport {
        target: controlling.target(),
        controlling,
        acted_at: now,
        controlled: done,
    }
}

fn one(key: &str, controlling: Controlling, reading: Option<&Snapshot>) -> Controlled {
    let target = controlling.target();
    if !target.holds(key) {
        return Controlled::refused(
            key,
            format!(
                "{key} is not a {}: this way is asked for from the list the rows of it are on",
                target.named()
            ),
        );
    }
    let Some(reading) = reading else {
        return Controlled::refused(key, NOT_READ);
    };
    let Some(item) = reading.items.get(key) else {
        return Controlled::refused(
            key,
            "no longer in the last reading: it changed or went away since it was marked, and \
             acting on the key alone would act on whatever took its place",
        );
    };

    match target {
        ControlTarget::Unit => match names::unit(key, item) {
            Err(why) => Controlled::refused(key, why),
            Ok(unit) => match systemctl::run(controlling, &unit) {
                Ok(said) => Controlled::done(key, Some(unit), said),
                Err(said) => Controlled::refused(key, said).about(Some(unit)),
            },
        },
        ControlTarget::Cron => match CronJob::of(item) {
            None => Controlled::refused(
                key,
                "the reading of this job holds no file, account, schedule and command, so \
                 there is no line to look for",
            ),
            Some(job) => {
                let source = job.source.clone();
                match cron::edit(&job, controlling == Controlling::Comment) {
                    Ok(said) => Controlled::done(key, Some(source), said),
                    Err(said) => Controlled::refused(key, said).about(Some(source)),
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Rfc3339 {
        "2026-09-16T10:00:00.000Z".into()
    }

    #[test]
    fn a_row_named_twice_in_one_ask_is_acted_on_once() {
        let mut made = 0;

        let report = asked(
            &["unit|nginx.service".into(), "unit|nginx.service".into()],
            Controlling::Stop,
            now(),
            &mut |key| {
                made += 1;
                Controlled::done(key, None, "done")
            },
        );

        assert_eq!(
            made, 1,
            "the same unit can be marked on the list and under the cursor, and stopping it \
             twice is a second stop nobody asked for"
        );
        assert_eq!(report.controlled.len(), 1);
    }

    #[test]
    fn an_ask_longer_than_the_ceiling_is_cut_and_every_row_past_it_says_so() {
        let many: Vec<String> = (0..MOST_AT_ONCE + 2)
            .map(|at| format!("unit|thing{at}.service"))
            .collect();

        let report = asked(&many, Controlling::Stop, now(), &mut |key| {
            Controlled::done(key, None, "done")
        });

        assert_eq!(report.controlled.len(), many.len());
        assert_eq!(report.done(), MOST_AT_ONCE);
        assert!(
            report.controlled[MOST_AT_ONCE].said.contains("Mark fewer"),
            "a row silently dropped is a row the operator believes is stopped: {}",
            report.controlled[MOST_AT_ONCE].said
        );
    }

    #[test]
    fn with_no_reading_behind_it_the_agent_touches_nothing_and_says_why_on_every_row() {
        for (key, controlling) in [
            ("unit|nginx.service", Controlling::Stop),
            (
                "cron|/etc/crontab|root|/usr/bin/backup",
                Controlling::Comment,
            ),
        ] {
            let report = carry_out(&[key.into()], controlling, None, now());

            assert_eq!(report.done(), 0);
            assert_eq!(report.target, controlling.target());
            assert!(
                report.controlled[0].said.contains("has not read"),
                "{:?}",
                report.controlled[0]
            );
        }
    }

    #[test]
    fn a_row_of_the_other_band_is_refused_by_name_before_any_program_is_started() {
        let report = carry_out(
            &["cron|/etc/crontab|root|/usr/bin/backup".into()],
            Controlling::Stop,
            None,
            now(),
        );

        assert_eq!(report.done(), 0);
        assert!(
            report.controlled[0].said.contains("is not a unit"),
            "a crontab line handed to systemctl is a unit name made of a whole command \
             line: {:?}",
            report.controlled[0]
        );
    }

    #[test]
    fn a_row_that_is_not_in_the_last_reading_is_refused_rather_than_acted_on_by_its_key() {
        let reading = Snapshot::new(READING, now());

        let report = carry_out(
            &["unit|nginx.service".into()],
            Controlling::Stop,
            Some(&reading),
            now(),
        );

        assert_eq!(report.done(), 0);
        assert!(
            report.controlled[0]
                .said
                .contains("no longer in the last reading"),
            "{:?}",
            report.controlled[0]
        );
    }

    #[test]
    fn what_starts_by_itself_is_looked_up_in_the_reading_that_holds_it() {
        assert_eq!(READING, "persistence");
    }
}
