use vigil_collect::KnownCollector;

use super::registry::modules;

pub fn watched() -> Vec<KnownCollector> {
    modules()
        .iter()
        .map(|module| KnownCollector {
            name: module.name(),
            subject: module.subject(),
            every_seconds: module.every_seconds(),
            unit: module.unit(),
        })
        .collect()
}

pub fn names() -> Vec<&'static str> {
    watched().into_iter().map(|watched| watched.name).collect()
}

pub fn is_known(name: &str) -> bool {
    watched().iter().any(|watched| watched.name == name)
}

pub fn subject_of(name: &str) -> Option<&'static str> {
    watched()
        .into_iter()
        .find(|watched| watched.name == name)
        .map(|watched| watched.subject)
}

pub fn every_seconds_of(name: &str) -> Option<u32> {
    watched()
        .into_iter()
        .find(|watched| watched.name == name)
        .map(|watched| watched.every_seconds)
}

pub fn unit_of(name: &str) -> Option<&'static str> {
    watched()
        .into_iter()
        .find(|watched| watched.name == name)
        .and_then(|watched| watched.unit)
}
