use vigil_model::{Envelope, Finding, Host, Producer, SCHEMA_VERSION};
use vigil_report::Reporter;

use crate::helpers::{rfc3339, uuid7};
use crate::socket::Shared;

pub struct Delivery {
    host: Host,
    reporters: Vec<Box<dyn Reporter>>,
    shared: Shared,
}

impl Delivery {
    pub fn new(host: Host, reporters: Vec<Box<dyn Reporter>>, shared: Shared) -> Self {
        Delivery {
            host,
            reporters,
            shared,
        }
    }

    pub fn send(&self, findings: &[Finding]) {
        if findings.is_empty() || self.reporters.is_empty() {
            return;
        }

        for reporter in &self.reporters {
            let wanted: Vec<Finding> = findings
                .iter()
                .filter(|finding| reporter.wants(finding))
                .cloned()
                .collect();
            if wanted.is_empty() {
                continue;
            }

            let envelope = Envelope {
                schema_version: SCHEMA_VERSION,
                batch_id: uuid7::mint(),
                sent_at: rfc3339::now(),
                producer: Producer {
                    name: "vigil".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                },
                host: self.host.clone(),
                findings: wanted,
            };

            let outcome = reporter
                .send(&envelope)
                .err()
                .map(|error| error.to_string());
            if let Some(error) = &outcome {
                eprintln!("reporter {} failed: {error}", reporter.name());
            }
            self.shared
                .with(|state| state.record_delivery(reporter.name(), rfc3339::now(), outcome));
        }
    }
}
