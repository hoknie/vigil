use vigil_model::{Finding, Kind};
use vigil_store::{Recorded, Store};

use crate::helpers::rfc3339;
use crate::types::Verdict;

use super::Round;

impl Round {
    pub(super) fn remember(&mut self, findings: &[Finding]) -> Vec<Finding> {
        let mut fresh = Vec::new();

        for finding in findings {
            self.close_what_it_ends(finding);

            let now = rfc3339::now();
            match self.policy.judge(finding, &now) {
                Verdict::Report => {}
                Verdict::Suppressed => {
                    let _ = self.store.record(finding);
                    continue;
                }
            }

            match self.store.record(finding) {
                Ok(Recorded::Created) => fresh.push(finding.clone()),
                Ok(Recorded::Repeated {
                    occurrences,
                    first_seen_at,
                }) => {
                    eprintln!(
                        "{} {} seen again ({occurrences} times since {first_seen_at})",
                        rfc3339::now(),
                        finding.finding_key
                    );
                }
                Err(error) => {
                    eprintln!(
                        "{} history not kept for {}: {error}",
                        rfc3339::now(),
                        finding.finding_key
                    );
                    fresh.push(finding.clone());
                }
            }
        }

        fresh
    }

    fn close_what_it_ends(&self, finding: &Finding) {
        let Kind::Known(kind) = &finding.kind else {
            return;
        };
        let Some(ended) = kind.resolves() else {
            return;
        };

        match self
            .store
            .resolve(&finding.finding_key, ended.as_str(), &finding.observed_at)
        {
            Ok(true) => eprintln!(
                "{} {} resolved: {}",
                rfc3339::now(),
                finding.finding_key,
                ended.as_str()
            ),
            Ok(false) => {}
            Err(error) => eprintln!(
                "{} could not resolve {}: {error}",
                rfc3339::now(),
                finding.finding_key
            ),
        }
    }
}
