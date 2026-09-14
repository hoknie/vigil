use vigil_store::Store;

use crate::helpers::rfc3339;

use super::Round;

impl Round {
    pub(super) fn take_what_the_console_did(&mut self) {
        let raised = self
            .shared
            .with(|state| state.take_what_the_console_raised());
        if raised.is_empty() {
            return;
        }

        for finding in &raised {
            eprintln!(
                "{} [{}] {} — {}",
                rfc3339::now(),
                finding.severity,
                finding.kind,
                finding.title
            );
            if let Err(error) = self.store.record(finding) {
                eprintln!(
                    "{} history not kept for {}: {error}",
                    rfc3339::now(),
                    finding.finding_key
                );
            }
        }

        self.hand_over(&raised);
    }
}
