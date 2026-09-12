use serde_json::json;
use vigil_model::Snapshot;

const AT: &str = "2026-09-09T09:00:00.000Z";

pub fn resources() -> Snapshot {
    Snapshot::new("resources", AT)
        .with(
            "boot|current",
            json!({
                "boot_id": "1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31",
                "booted_at": 1_757_419_200,
                "readable": true,
            }),
        )
        .with(
            "memory|summary",
            json!({
                "total_bytes": 8_232_091_648u64,
                "swap_total_bytes": 1_073_737_728u64,
                "readable": true,
            }),
        )
        .with(
            "fs|/",
            json!({
                "mount": "/", "device": "/dev/sda1", "type": "ext4",
                "free_percent_step": 60, "free_inodes_percent_step": 85,
                "total_bytes": 52_479_123_456u64, "read_only": false,
            }),
        )
        .with(
            "fs|/var",
            json!({
                "mount": "/var", "device": "/dev/sda2", "type": "ext4",
                "free_percent_step": 5, "free_inodes_percent_step": 85,
                "total_bytes": 20_938_809_344u64, "read_only": false,
            }),
        )
        .with(
            "fs|/run",
            json!({
                "mount": "/run", "device": "tmpfs", "type": "tmpfs",
                "free_percent_step": 95, "free_inodes_percent_step": 95,
                "total_bytes": 821_264_384u64, "read_only": true,
            }),
        )
}
