use vigil_model::{Counted, StoreDropped, StoreStatus};

pub fn store() -> StoreStatus {
    StoreStatus {
        records: Counted {
            held: 1_284,
            ceiling: 10_000,
        },
        bytes: Counted {
            held: 2_202_009,
            ceiling: 16 * 1024 * 1024,
        },
        oldest_at: Some("2026-08-27T04:11:53.000Z".into()),
        dropped: StoreDropped {
            at_the_ceiling: 0,
            past_the_window: 41,
        },
        damaged: 3,
        journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
    }
}
