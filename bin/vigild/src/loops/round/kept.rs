use vigil_model::{Counted, StoreDropped, StoreStatus};
use vigil_store::Store;

use crate::helpers::rfc3339;

use super::Round;

impl Round {
    pub(super) fn take_store(&self) {
        let kept = match self.store.kept() {
            Ok(kept) => kept,
            Err(error) => {
                eprintln!("{} store not counted: {error}", rfc3339::now());
                return;
            }
        };

        let status = StoreStatus {
            records: Counted {
                held: kept.records.held,
                ceiling: kept.records.ceiling,
            },
            bytes: Counted {
                held: kept.bytes.held,
                ceiling: kept.bytes.ceiling,
            },
            oldest_at: kept.oldest_at,
            dropped: StoreDropped {
                at_the_ceiling: kept.dropped.at_the_ceiling,
                past_the_window: kept.dropped.past_the_window,
            },
            damaged: kept.damaged,
            journal_path: self
                .store
                .journal_path()
                .map(|path| path.display().to_string()),
        };

        self.shared.with(|state| state.record_store(status));
    }
}
