use std::io::Write;
use std::sync::Mutex;

use vigil_model::Envelope;

use crate::{Delivery, ReportError, Reporter};

pub struct NdjsonSink<W: Write + Send> {
    name: String,
    out: Mutex<W>,
}

impl<W: Write + Send> NdjsonSink<W> {
    pub fn new(name: impl Into<String>, out: W) -> Self {
        NdjsonSink {
            name: name.into(),
            out: Mutex::new(out),
        }
    }
}

impl<W: Write + Send> Reporter for NdjsonSink<W>
where
    W: Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn send(&self, envelope: &Envelope) -> Result<Delivery, ReportError> {
        let line = serde_json::to_string(envelope)
            .map_err(|e| ReportError::Malformed(format!("envelope not serialised: {e}")))?;

        let mut out = self
            .out
            .lock()
            .map_err(|_| ReportError::Transient("writer poisoned by a panic".into()))?;
        writeln!(out, "{line}").map_err(|e| ReportError::Transient(e.to_string()))?;
        out.flush()
            .map_err(|e| ReportError::Transient(e.to_string()))?;

        Ok(Delivery::Accepted {
            accepted: envelope.findings.len() as u64,
            duplicates: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::{Envelope, Host, Os, Producer, SCHEMA_VERSION};

    use super::*;

    fn envelope() -> Envelope {
        Envelope {
            schema_version: SCHEMA_VERSION,
            batch_id: "0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e1".into(),
            sent_at: "2026-09-08T12:04:02.311Z".into(),
            producer: Producer {
                name: "vigil".into(),
                version: "0.1.0".into(),
            },
            host: Host {
                host_id: "b7f1c0".into(),
                install_id: "0192e7aa-1c02-7f10-8d44-9a1b6c2e5f03".into(),
                boot_id: "6f2c".into(),
                hostname: "web-03".into(),
                fqdn: None,
                os: Os {
                    family: "linux".into(),
                    distro: "debian".into(),
                    version: "12".into(),
                    kernel: "6.1.0-18-amd64".into(),
                    arch: "x86_64".into(),
                },
                addresses: vec!["10.0.0.13".into()],
                tags: Default::default(),
                peer: None,
            },
            findings: Vec::new(),
        }
    }

    #[test]
    fn writes_one_line_per_envelope_and_names_the_version_on_the_wire() {
        let sink = NdjsonSink::new("file", Vec::new());

        sink.send(&envelope()).expect("writes");
        sink.send(&envelope()).expect("writes again");

        let written = sink.out.lock().expect("not poisoned").clone();
        let text = String::from_utf8(written).expect("utf-8");
        assert_eq!(text.lines().count(), 2);
        assert!(
            text.contains(&format!(r#""schema_version":"{SCHEMA_VERSION}""#)),
            "the version travels as text, not as two integers: {text}"
        );
    }
}
