use serde_json::Value;
use vigil_rules::{is_a_path_of_this_host, is_a_path_of_this_host_whatever_follows};

use super::row::{field_list, field_text};
use crate::types::{Engine, Subject};

const CUT_SHORT: char = '\u{2026}';

pub fn held_of_this_host(engine: Engine, subject: Subject, item: &Value) -> Vec<&str> {
    match (engine, subject) {
        (_, Subject::Volume) => field_text(item, "device")
            .filter(|device| is_a_path_of_this_host(device))
            .into_iter()
            .collect(),
        (Engine::Docker, Subject::Container) => field_list(item, "mounts")
            .into_iter()
            .filter(|mount| mount.starts_with('/'))
            .filter(|mount| match mount.split_once(CUT_SHORT) {
                Some((visible, _)) => is_a_path_of_this_host_whatever_follows(visible),
                None => is_a_path_of_this_host(mount),
            })
            .collect(),
        _ => Vec::new(),
    }
}
