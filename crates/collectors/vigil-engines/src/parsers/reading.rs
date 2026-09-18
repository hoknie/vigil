use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::printed::{alone, listed};
use super::subjects::{
    containers, images, information, networks, pods, projects, registries, secrets, volumes,
};
use crate::types::{Answer, Dump, Engine, Registry, Subject};

pub const SOURCE: &str = "containers-engines";

pub struct EngineReading<'a> {
    pub engine: Engine,
    pub dump: Option<&'a Dump>,
    pub registries: Vec<Registry>,
    pub registries_read: bool,
}

pub struct EnginesReading<'a> {
    pub engines: &'a [EngineReading<'a>],
}

pub fn engines_snapshot(taken_at: &str, reading: &EnginesReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    for read in reading.engines {
        for (subject, rows) in of_one(read) {
            for (id, item) in rows {
                snapshot.items.insert(key(read.engine, subject, &id), item);
            }
        }
    }

    snapshot
}

pub fn key(engine: Engine, subject: Subject, id: &str) -> String {
    format!("{}|{}|{}", engine.name(), subject.as_str(), id)
}

fn of_one(read: &EngineReading<'_>) -> Vec<(Subject, BTreeMap<String, Value>)> {
    let present = read.dump.is_some_and(Dump::on_this_host);
    let itself = read.dump.and_then(said_about_itself);
    let mut information = information(read.engine, present, itself.as_ref());
    information["dump_read"] = json!(read.dump.is_some());
    information["unanswered"] = json!(unanswered(read));

    let mut gathered = vec![
        (
            Subject::Engine,
            BTreeMap::from([(read.engine.name().to_string(), information)]),
        ),
        (Subject::Registry, registries(&read.registries)),
    ];

    let Some(dump) = read.dump.filter(|dump| dump.on_this_host()) else {
        return gathered;
    };

    let rows = |subject: Subject| -> Vec<Value> {
        match dump
            .answer(subject.as_str())
            .filter(|answer| answer.answered())
        {
            Some(answer) => listed(&answer.printed),
            None => Vec::new(),
        }
    };

    let held = containers(&rows(Subject::Container));
    gathered.push((Subject::Image, images(&rows(Subject::Image))));
    gathered.push((Subject::Volume, volumes(&rows(Subject::Volume))));
    gathered.push((Subject::Network, networks(&rows(Subject::Network))));
    gathered.push((Subject::Project, projects(&rows(Subject::Container), &held)));
    gathered.push((Subject::Container, held));
    gathered.push((Subject::Pod, pods(&rows(Subject::Pod))));
    gathered.push((Subject::Secret, secrets(&rows(Subject::Secret))));

    gathered
}

fn unanswered(read: &EngineReading<'_>) -> Vec<&'static str> {
    let mut silent: Vec<&'static str> = match read.dump.filter(|dump| dump.on_this_host()) {
        Some(dump) => read
            .engine
            .asks()
            .iter()
            .map(|asked| asked.subject)
            .filter(|subject| *subject != Subject::Engine)
            .filter(|subject| !dump.answer(subject.as_str()).is_some_and(Answer::answered))
            .map(Subject::as_str)
            .collect(),
        None => Vec::new(),
    };
    if !read.registries_read {
        silent.push(Subject::Registry.as_str());
    }
    silent.sort_unstable();
    silent
}

fn said_about_itself(dump: &Dump) -> Option<Value> {
    let answer = dump
        .answer(Subject::Engine.as_str())
        .filter(|answer| answer.answered())?;

    alone(&answer.printed)
}
