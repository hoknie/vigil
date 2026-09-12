use vigil_model::Finding;

use crate::helpers::rfc3339;

use super::Round;

impl Round {
    pub(super) fn take_buffers(&self) {
        let buffers = self.delivery.buffers();
        self.shared.with(|state| state.record_buffers(buffers));
    }

    pub(super) fn hand_over(&mut self, findings: &[Finding]) {
        let raised = self.delivery.send(findings);
        self.take_buffers();
        if raised.is_empty() {
            return;
        }

        let fresh = self.remember(&raised);
        for finding in &fresh {
            eprintln!(
                "{} [{}] {} — {}",
                rfc3339::now(),
                finding.severity,
                finding.kind,
                finding.title
            );
        }
        self.shared.with(|state| state.record_findings(&fresh));
        self.delivery.send(&fresh);
        self.take_buffers();
    }
}
