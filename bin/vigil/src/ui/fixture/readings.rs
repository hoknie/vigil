use serde_json::json;
use vigil_model::Snapshot;

const AT: &str = "2026-09-09T09:00:00.000Z";

pub fn with_no_screen() -> Snapshot {
    Snapshot::new("kernel", AT)
        .with(
            "module|overlay",
            json!({
                "name": "overlay",
                "size": 172_032,
                "used_by": 3,
                "signed": true,
                "tainted": false,
            }),
        )
        .with(
            "module|vboxdrv",
            json!({
                "name": "vboxdrv",
                "size": 651_264,
                "used_by": 0,
                "signed": false,
                "tainted": true,
            }),
        )
}
