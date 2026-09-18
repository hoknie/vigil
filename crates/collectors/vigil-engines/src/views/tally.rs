use vigil_model::Snapshot;
use vigil_view::{Counts, Showing, time_of_day};

use super::facts::{in_use, of_the_list, said_by_the_engine};
use crate::helpers::{field_flag, field_list, held_of_this_host, untagged_image};
use crate::types::{Engine, List, Standing};

const HELD: &str = "held";
const ONE: &str = "one";
const TWO: &str = "two";
const THREE: &str = "three";
const ENGINE: &str = "engine";
const SILENT: &str = "silent";

pub(super) fn counts(reading: &Snapshot, engine: Engine, list: List) -> Counts {
    let mut held = 0;
    let mut numbers = [0usize; 3];
    for (key, item) in of_the_list(reading, engine, list) {
        held += 1;
        let said: [bool; 3] = match list {
            List::Containers => [
                field_flag(item, "host_network"),
                !held_of_this_host(engine, list.subject(), item).is_empty(),
                untagged_image(item).is_some(),
            ],
            List::Images => [
                field_flag(item, "untagged"),
                in_use(reading, engine, key, item) == 0,
                false,
            ],
            List::Volumes => [
                !held_of_this_host(engine, list.subject(), item).is_empty(),
                false,
                false,
            ],
            List::Networks => [
                field_flag(item, "internal"),
                field_list(item, "subnets").is_empty(),
                false,
            ],
            List::Registries => [field_flag(item, "insecure"), false, false],
            _ => [false; 3],
        };
        for (at, yes) in said.into_iter().enumerate() {
            numbers[at] += usize::from(yes);
        }
    }

    let silent = match Standing::in_reading(reading, engine).silent_on(list.subject()) {
        true => vec![format!(
            "{} did not answer for its {}",
            engine.name(),
            list.name()
        )],
        false => Vec::new(),
    };

    Counts::default()
        .counted(HELD, held)
        .counted(ONE, numbers[0])
        .counted(TWO, numbers[1])
        .counted(THREE, numbers[2])
        .saying(ENGINE, said_by_the_engine(reading, engine))
        .saying(SILENT, silent)
}

pub(super) fn tallied(
    reading: &Snapshot,
    engine: Engine,
    list: List,
    showing: &Showing<'_>,
    shown: usize,
    counts: &Counts,
) -> String {
    let read_at = time_of_day(&reading.taken_at);
    let held = counts.number(HELD);
    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{shown} {} of {}, read at {read_at}",
            counted(list, shown),
            engine.name()
        ),
        true => format!(
            "{shown} of {held} {} of {}, read at {read_at}",
            counted(list, held),
            engine.name()
        ),
    }];
    let narrowed: Vec<String> = showing
        .only
        .iter()
        .map(|facet| format!("{} {}", facet.name, facet.value))
        .collect();
    if !narrowed.is_empty() {
        parts.push(format!("only {}", narrowed.join(" and ")));
    }
    parts.extend(counts.words(SILENT).iter().cloned());
    let facts = counts.words(ENGINE);
    if !facts.is_empty() {
        parts.push(facts.join(", "));
    }
    for (name, said) in [
        (ONE, first(list)),
        (TWO, second(list)),
        (THREE, third(list)),
    ] {
        let number = counts.number(name);
        if number > 0 && !said.is_empty() {
            parts.push(format!("{number} {said}"));
        }
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
            showing.elsewhere
        ));
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" \u{b7} ")
}

fn counted(list: List, number: usize) -> String {
    match (list, number) {
        (List::Registries, 1) => "registry".to_string(),
        (List::Registries, _) => "registries".to_string(),
        (other, _) => format!("{}(s)", other.thing()),
    }
}

fn first(list: List) -> &'static str {
    match list {
        List::Containers => "on the host network",
        List::Images => "untagged",
        List::Volumes => "binding a path of this host",
        List::Networks => "internal",
        List::Registries => "without TLS",
        _ => "",
    }
}

fn second(list: List) -> &'static str {
    match list {
        List::Containers => "mounting paths of this host",
        List::Images => "run by no container",
        List::Networks => "with no subnet printed",
        _ => "",
    }
}

fn third(list: List) -> &'static str {
    match list {
        List::Containers => "running an untagged image",
        _ => "",
    }
}
