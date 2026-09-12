use serde_json::Value;

use super::kind::Kind;
use super::row::Row;
use crate::ui::helpers::words::{moment, size};

const UNSAID: &str = "—";

pub fn what(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Boot => "this host's boot".to_string(),
        Kind::Memory => "memory and swap".to_string(),
        Kind::Filesystem => text(row.item, "mount").to_string(),
    }
}

pub fn free(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Filesystem => percent(row.item, "free_percent_step"),
        _ => UNSAID.to_string(),
    }
}

pub fn free_inodes(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Filesystem => percent(row.item, "free_inodes_percent_step"),
        _ => UNSAID.to_string(),
    }
}

pub fn held(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Filesystem => size::bytes(number(row.item, "total_bytes")),
        Kind::Memory => size::bytes(number(row.item, "total_bytes")),
        Kind::Boot => UNSAID.to_string(),
    }
}

pub fn kind_of_store(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Filesystem => text(row.item, "type").to_string(),
        Kind::Memory => format!("swap {}", size::bytes(number(row.item, "swap_total_bytes"))),
        Kind::Boot => booted(row.item),
    }
}

pub fn device(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Filesystem => text(row.item, "device").to_string(),
        Kind::Boot => short(text(row.item, "boot_id")).to_string(),
        Kind::Memory => UNSAID.to_string(),
    }
}

pub fn how_it_is_mounted(row: &Row<'_>) -> String {
    match (row.kind, row.item["read_only"].as_bool()) {
        (Kind::Filesystem, Some(true)) => "read only".to_string(),
        (Kind::Filesystem, _) => "read write".to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn booted(item: &Value) -> String {
    match item["booted_at"].as_i64() {
        Some(seconds) => moment::utc(seconds),
        None => UNSAID.to_string(),
    }
}

pub fn readable(item: &Value) -> bool {
    item["readable"].as_bool().unwrap_or(true)
}

pub fn free_step(item: &Value) -> Option<u64> {
    item["free_percent_step"].as_u64()
}

pub fn free_inodes_step(item: &Value) -> Option<u64> {
    item["free_inodes_percent_step"].as_u64()
}

pub fn sort_key(row: &Row<'_>) -> (Kind, String) {
    (row.kind, what(row))
}

pub fn short(boot_id: &str) -> &str {
    match boot_id.char_indices().nth(12) {
        Some((at, _)) => &boot_id[..at],
        None => boot_id,
    }
}

pub fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap_or("?")
}

pub fn number(item: &Value, field: &str) -> u64 {
    item[field].as_u64().unwrap_or(0)
}

fn percent(item: &Value, field: &str) -> String {
    match item[field].as_u64() {
        Some(step) => format!("{step}%"),
        None => UNSAID.to_string(),
    }
}

pub fn means(row: &Row<'_>) -> Vec<String> {
    match row.kind {
        Kind::Boot => match readable(row.item) {
            true => vec![format!("This host was booted at {}.", booted(row.item))],
            false => vec![
                "The kernel's boot id could not be read on this host, so a reboot between two \
                 readings cannot be told from a host that stayed up."
                    .to_string(),
            ],
        },
        Kind::Memory => match readable(row.item) {
            true => Vec::new(),
            false => vec![
                "How much memory this host has could not be read, so the size beside it is \
                 not what the kernel says."
                    .to_string(),
            ],
        },
        Kind::Filesystem => match row.item["read_only"].as_bool() {
            Some(true) => vec![
                "This filesystem is mounted read only: nothing writes to it until something \
                 mounts it again."
                    .to_string(),
            ],
            _ => Vec::new(),
        },
    }
}
