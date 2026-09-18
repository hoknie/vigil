use vigil_model::class_of;

use crate::fixture::{self, docker, podman};
use crate::parsers::SOURCE;
use crate::types::{Engine, Subject};

#[test]
fn every_row_is_keyed_by_the_engine_then_the_subject_then_the_thing_itself() {
    let reading = fixture::engines();

    assert_eq!(reading.source, SOURCE);
    for (key, item) in &reading.items {
        let parts: Vec<&str> = key.splitn(3, '|').collect();
        assert_eq!(parts.len(), 3, "{key} is not <engine>|<subject>|<id>");
        assert!(Engine::named(parts[0]).is_some(), "{key}");
        assert_eq!(
            Subject::named(parts[1]).map(Subject::as_str),
            item["subject"].as_str(),
            "{key} is filed under one subject and says it is another"
        );
        assert!(!parts[2].is_empty(), "{key} names nothing");
    }
}

#[test]
fn the_engine_is_the_first_part_of_the_key_so_that_a_class_is_an_engine_of_this_host() {
    let reading = fixture::engines();

    let mut classes: Vec<&str> = reading.items.keys().map(|key| class_of(key)).collect();
    classes.sort_unstable();
    classes.dedup();

    assert_eq!(
        classes,
        vec!["docker", "podman"],
        "the first row of the containers menu is the engine, and the class of a row is what \
         that row is filed under: keying by the subject first would put the two engines' \
         images in one list and the menu would have nothing to select on"
    );
}

#[test]
fn two_readings_of_one_dump_are_the_same_reading_down_to_the_value() {
    let first = fixture::engines();
    let again = fixture::engines();

    assert_eq!(
        first.items, again.items,
        "a reading that differs from itself is a finding on every round, and a host that \
         reports on every round is a host nobody reads"
    );
}

#[test]
fn nothing_that_moves_while_this_host_stands_still_reaches_the_reading() {
    let before = fixture::engines();
    let after = fixture::read(&aged(&docker::dump()), &aged(&podman::dump()));

    assert_eq!(
        before.items, after.items,
        "uptime, a status line, a restart counter and `5 minutes ago` all move on a host \
         where nothing happened. A reading carrying one of them makes a finding out of the \
         clock"
    );
}

#[test]
fn the_time_the_dump_took_and_the_hour_it_was_taken_at_are_not_part_of_the_reading() {
    let mut later = docker::dump();
    for answer in later.asked.values_mut() {
        answer.milliseconds += 1_000;
    }
    later.taken_at = "2026-09-17T09:02:01.000Z".to_string();

    assert_eq!(
        fixture::read(&docker::dump(), &podman::dump()).items,
        fixture::read(&later, &podman::dump()).items,
        "what the dump cost and when it ran belong to the dump, not to what the host holds"
    );
}

fn aged(dump: &crate::types::Dump) -> crate::types::Dump {
    let mut moved = dump.clone();

    for answer in moved.asked.values_mut() {
        answer.printed = answer
            .printed
            .replace("Up 16 days", "Up 16 days (healthy)")
            .replace("Up 6 days", "Up 7 days")
            .replace("20 hours ago", "21 hours ago")
            .replace("3 days ago", "4 days ago")
            .replace("\"Restarts\":3", "\"Restarts\":9")
            .replace("\"Containers\":3", "\"Containers\":4")
            .replace("\"uptime\":\"146h 3m 11s\"", "\"uptime\":\"147h 0m 4s\"")
            .replace("09:00:01.412339Z", "10:00:01.998877Z");
    }

    moved
}

#[test]
fn a_subject_the_engine_did_not_answer_for_has_no_rows_and_the_engine_row_names_it() {
    let reading = fixture::docker_silent_on(Subject::Volume);
    let engine = fixture::row(&reading, Engine::Docker, Subject::Engine, "docker");

    assert!(
        !reading
            .items
            .keys()
            .any(|key| key.starts_with("docker|volume|")),
        "a command that failed printed nothing, and nothing is what the reading holds for it"
    );
    assert_eq!(
        engine["unanswered"],
        serde_json::json!(["volume"]),
        "the engine row is the one place that tells a subject with no rows from a subject the \
         engine never answered for: without it the rules read a failed command as every \
         volume removed, and the screen reads it as an engine that holds none"
    );
    assert_eq!(engine["dump_read"], serde_json::json!(true));
}

#[test]
fn an_engine_that_answered_everything_names_nothing_it_did_not_answer_for() {
    let reading = fixture::engines();

    for engine in Engine::ALL {
        let row = fixture::row(&reading, engine, Subject::Engine, engine.name());
        assert_eq!(
            row["unanswered"],
            serde_json::json!([]),
            "{}",
            engine.name()
        );
        assert_eq!(
            row["dump_read"],
            serde_json::json!(true),
            "{}",
            engine.name()
        );
    }
}
