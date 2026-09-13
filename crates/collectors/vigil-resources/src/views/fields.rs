use serde_json::Value;
use vigil_view::{bytes, utc};

use crate::types::Family;

const UNSAID: &str = "—";

pub fn what(family: Family, item: &Value) -> String {
    match family {
        Family::Boot => "this host's boot".to_string(),
        Family::Memory => "memory and swap".to_string(),
        Family::Filesystem => text(item, "mount").to_string(),
    }
}

pub fn free(family: Family, item: &Value) -> String {
    match family {
        Family::Filesystem => percent(item, "free_percent_step"),
        _ => UNSAID.to_string(),
    }
}

pub fn free_inodes(family: Family, item: &Value) -> String {
    match family {
        Family::Filesystem => percent(item, "free_inodes_percent_step"),
        _ => UNSAID.to_string(),
    }
}

pub fn held(family: Family, item: &Value) -> String {
    match family {
        Family::Filesystem | Family::Memory => bytes(number(item, "total_bytes")),
        Family::Boot => UNSAID.to_string(),
    }
}

pub fn kind_of_store(family: Family, item: &Value) -> String {
    match family {
        Family::Filesystem => text(item, "type").to_string(),
        Family::Memory => format!("swap {}", bytes(number(item, "swap_total_bytes"))),
        Family::Boot => booted(item),
    }
}

pub fn device(family: Family, item: &Value) -> String {
    match family {
        Family::Filesystem => text(item, "device").to_string(),
        Family::Boot => short(text(item, "boot_id")).to_string(),
        Family::Memory => UNSAID.to_string(),
    }
}

pub fn how_it_is_mounted(family: Family, item: &Value) -> String {
    match (family, item["read_only"].as_bool()) {
        (Family::Filesystem, Some(true)) => "read only".to_string(),
        (Family::Filesystem, _) => "read write".to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn booted(item: &Value) -> String {
    match item["booted_at"].as_i64() {
        Some(seconds) => utc(seconds),
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

pub fn sort_key(family: Family, item: &Value) -> (Family, String) {
    (family, what(family, item))
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

pub fn means(family: Family, item: &Value) -> Vec<String> {
    match family {
        Family::Boot => match readable(item) {
            true => vec![format!("This host was booted at {}.", booted(item))],
            false => vec![
                "The kernel's boot id could not be read on this host, so a reboot between two \
                 readings cannot be told from a host that stayed up."
                    .to_string(),
            ],
        },
        Family::Memory => match readable(item) {
            true => Vec::new(),
            false => vec![
                "How much memory this host has could not be read, so the size beside it is \
                 not what the kernel says."
                    .to_string(),
            ],
        },
        Family::Filesystem => match item["read_only"].as_bool() {
            Some(true) => vec![
                "This filesystem is mounted read only: nothing writes to it until something \
                 mounts it again."
                    .to_string(),
            ],
            _ => Vec::new(),
        },
    }
}
