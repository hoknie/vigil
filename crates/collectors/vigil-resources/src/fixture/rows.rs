use serde_json::{Value, json};

pub fn boot(boot_id: &str, booted_at: i64) -> Value {
    json!({
        "boot_id": boot_id,
        "booted_at": booted_at,
        "readable": true,
    })
}

pub fn boot_unreadable() -> Value {
    json!({
        "boot_id": Value::Null,
        "booted_at": Value::Null,
        "readable": false,
    })
}

pub fn memory(total_bytes: u64, swap_total_bytes: Option<u64>) -> Value {
    json!({
        "total_bytes": total_bytes,
        "swap_total_bytes": swap_total_bytes,
        "readable": true,
    })
}

pub fn filesystem(mount: &str, free_percent_step: Option<u64>, free_inodes: Option<u64>) -> Value {
    json!({
        "mount": mount,
        "device": "/dev/sda2",
        "type": "ext4",
        "storage": "sda",
        "storage_from": "disk",
        "read_only": false,
        "total_bytes": 20_938_809_344u64,
        "free_percent_step": free_percent_step,
        "free_inodes_percent_step": free_inodes,
    })
}
