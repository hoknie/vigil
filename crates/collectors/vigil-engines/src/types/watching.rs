use serde::Deserialize;

use super::engine::Engine;

pub const DUMP_SECONDS: u32 = 120;

const LONGEST_DUMP_SECONDS: u32 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Watching {
    pub engines: Vec<String>,
    pub dump_seconds: u32,
    pub report: Report,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Report {
    pub images: bool,
    pub volumes: bool,
    pub networks: bool,
    pub projects: bool,
    pub pods: bool,
    pub secrets: bool,
}

impl Default for Watching {
    fn default() -> Self {
        Watching {
            engines: Engine::ALL
                .into_iter()
                .map(|engine| engine.name().to_string())
                .collect(),
            dump_seconds: DUMP_SECONDS,
            report: Report::default(),
        }
    }
}

impl Default for Report {
    fn default() -> Self {
        Report {
            images: true,
            volumes: true,
            networks: true,
            projects: true,
            pods: true,
            secrets: true,
        }
    }
}

impl Watching {
    pub fn check(&self) -> Result<(), String> {
        if self.engines.is_empty() {
            return Err(
                "engines: an empty list reads no engine at all. Take the collector out \
                        of `collectors:` to mean that, and the console says so by name"
                    .to_string(),
            );
        }

        for (index, named) in self.engines.iter().enumerate() {
            if Engine::named(named).is_none() {
                return Err(format!(
                    "engines #{}: {named:?} is no engine this build reads; it has: {}",
                    index + 1,
                    known()
                ));
            }
            if self.engines.iter().filter(|other| *other == named).count() > 1 {
                return Err(format!("engines: {named:?} is named twice"));
            }
        }

        match self.dump_seconds {
            0 => Err("dump_seconds: 0 is a dump with no pause between dumps".to_string()),
            seconds if seconds > LONGEST_DUMP_SECONDS => Err(format!(
                "dump_seconds: {seconds} is longer than a day, and a reading older than twice \
                 this is reported as stale for the rest of it"
            )),
            _ => Ok(()),
        }
    }

    pub fn watched(&self) -> Vec<Engine> {
        self.engines
            .iter()
            .filter_map(|named| Engine::named(named))
            .collect()
    }

    pub fn stale_after_seconds(&self) -> u64 {
        u64::from(self.dump_seconds) * 2
    }
}

fn known() -> String {
    Engine::ALL
        .into_iter()
        .map(Engine::name)
        .collect::<Vec<&str>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn read(said: serde_json::Value) -> Result<Watching, String> {
        serde_json::from_value::<Watching>(said).map_err(|error| error.to_string())
    }

    #[test]
    fn a_block_with_nothing_in_it_reads_both_engines_every_two_minutes_and_reports_everything() {
        let watching = Watching::default();

        assert_eq!(watching.engines, vec!["docker", "podman"]);
        assert_eq!(watching.dump_seconds, DUMP_SECONDS);
        assert_eq!(watching.stale_after_seconds(), 240);
        assert_eq!(watching.report, Report::default());
        assert!(watching.report.images && watching.report.secrets);
        assert!(watching.check().is_ok());
    }

    #[test]
    fn an_engine_this_build_never_heard_of_is_refused_by_name_and_told_which_ones_exist() {
        let watching = read(json!({"engines": ["docker", "containerd"]})).expect("parses");

        let refusal = watching.check().expect_err("must not be accepted");

        assert!(refusal.contains("containerd"), "{refusal}");
        assert!(refusal.contains("podman"), "{refusal}");
    }

    #[test]
    fn a_key_inside_the_block_that_nobody_declares_is_refused_rather_than_read_as_silence() {
        let refusal = read(json!({"dump_secondss": 60})).expect_err("must not be ignored");

        assert!(refusal.contains("dump_secondss"), "{refusal}");
    }

    #[test]
    fn a_toggle_inside_report_that_nobody_declares_is_refused_by_the_block_that_owns_it() {
        let refusal = read(json!({"report": {"imagess": false}})).expect_err("must not be ignored");

        assert!(refusal.contains("imagess"), "{refusal}");
    }

    #[test]
    fn a_dump_period_of_zero_is_refused_rather_than_turned_into_a_default() {
        let refusal = read(json!({"dump_seconds": 0}))
            .expect("parses")
            .check()
            .expect_err("must not be accepted");

        assert!(refusal.contains("dump_seconds"), "{refusal}");
    }

    #[test]
    fn the_dump_is_called_stale_at_twice_the_period_it_is_written_at_and_not_at_one() {
        let watching = read(json!({"dump_seconds": 30})).expect("parses");

        assert_eq!(
            watching.stale_after_seconds(),
            60,
            "one period is the ordinary case: a dump written a moment before the reading is \
             the dump of the previous run for as long as the two are not in step"
        );
    }

    #[test]
    fn only_the_engines_named_are_read_and_a_name_given_twice_is_refused() {
        let one = read(json!({"engines": ["podman"]})).expect("parses");
        assert!(one.check().is_ok());
        assert_eq!(one.watched(), vec![Engine::Podman]);

        let twice = read(json!({"engines": ["docker", "docker"]})).expect("parses");
        assert!(twice.check().is_err());
    }
}
