use vigil_store::Store;

use crate::helpers::rfc3339;

use super::Round;

impl Round {
    pub(super) fn take_retention(&self) {
        match self.store.prune(&rfc3339::days_ago(self.retention_days)) {
            Ok(0) => return,
            Ok(dropped) => eprintln!(
                "{} history: {dropped} finding(s) past the retention window dropped",
                rfc3339::now()
            ),
            Err(error) => eprintln!(
                "{} history: could not apply retention: {error}",
                rfc3339::now()
            ),
        }
        self.take_store();
    }
}
