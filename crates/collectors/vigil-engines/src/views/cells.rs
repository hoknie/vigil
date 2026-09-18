use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::bytes;

use super::columns::every;
use super::facts::{id_of, in_use, short};
use crate::helpers::{field_flag, field_list, field_text, held_of_this_host, project_of};
use crate::types::{Engine, List};

const UNSAID: &str = "\u{2014}";

pub(super) struct Row<'a> {
    pub engine: Engine,
    pub list: List,
    pub key: &'a str,
    pub item: &'a Value,
    pub reading: &'a Snapshot,
}

pub(super) fn value(row: &Row<'_>, column: usize) -> String {
    let item = row.item;
    let header = every(row.list).get(column).map_or("", |shown| shown.header);
    match (row.list, header) {
        (_, "NAME" | "POD" | "SECRET" | "REGISTRY") => id_of(row.key).to_string(),
        (List::Compose, "PROJECT") => id_of(row.key).to_string(),
        (_, "PROJECT") => project_of(item).unwrap_or(UNSAID).to_string(),
        (_, "IMAGE") => text(item, "image"),
        (List::Pods, "CONTAINERS") => joined(item, "containers"),
        (_, "NETWORKS") => joined(item, "networks"),
        (_, "PORTS") => joined(item, "ports"),
        (_, "HOST PATHS") => match row.engine {
            Engine::Docker => held_of_this_host(row.engine, row.list.subject(), item)
                .len()
                .to_string(),
            Engine::Podman => UNSAID.to_string(),
        },
        (_, "TAGS") => match field_list(item, "tags").as_slice() {
            [] => "<none>".to_string(),
            tags => tags.join(", "),
        },
        (_, "ID") => short(id_of(row.key)),
        (_, "DIGEST") => field_text(item, "digest").map_or_else(|| UNSAID.to_string(), short),
        (_, "SIZE") => match field_text(item, "size").map(str::parse::<u64>) {
            Some(Ok(size)) => bytes(size),
            _ => text(item, "size"),
        },
        (_, "IN USE") => in_use(row.reading, row.engine, row.key, item).to_string(),
        (_, "DRIVER") => text(item, "driver"),
        (_, "BINDS") => text(item, "device"),
        (_, "MOUNTPOINT") => text(item, "mountpoint"),
        (_, "SUBNETS") => match (field_list(item, "subnets").is_empty(), row.engine) {
            (true, Engine::Docker) => "not printed".to_string(),
            _ => joined(item, "subnets"),
        },
        (_, "INTERNAL") => yes_or_no(item, "internal"),
        (_, "INTERFACE") => text(item, "interface"),
        (_, "SERVICES") => joined(item, "services"),
        (List::Compose, "CONTAINERS") => field_list(item, "containers").len().to_string(),
        (_, "DIRECTORY") => text(item, "working_directory"),
        (_, "CGROUP") => text(item, "cgroup"),
        (_, "CREATED") => text(item, "created_at"),
        (_, "UPDATED") => text(item, "updated_at"),
        (_, "TLS") => match field_flag(item, "insecure") {
            true => "no".to_string(),
            false => "yes".to_string(),
        },
        (_, "ROLE") => text(item, "role"),
        (_, "FROM THE FILE") => text(item, "from"),
        _ => UNSAID.to_string(),
    }
}

pub(super) fn ordered(row: &Row<'_>, column: usize) -> String {
    let said = value(row, column);
    match every(row.list)
        .get(column)
        .is_some_and(|shown| shown.counted)
    {
        true => format!("{:0>10}", said),
        false => said,
    }
}

fn text(item: &Value, field: &str) -> String {
    field_text(item, field).unwrap_or(UNSAID).to_string()
}

fn joined(item: &Value, field: &str) -> String {
    match field_list(item, field).as_slice() {
        [] => UNSAID.to_string(),
        listed => listed.join(", "),
    }
}

fn yes_or_no(item: &Value, field: &str) -> String {
    match field_flag(item, field) {
        true => "yes".to_string(),
        false => "no".to_string(),
    }
}
