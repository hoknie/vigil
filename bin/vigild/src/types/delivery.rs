use std::sync::Mutex;

use vigil_model::{BufferStatus, Envelope, Finding, Host, Producer, SCHEMA_VERSION};
use vigil_report::{ReportError, Reporter};
use vigil_store::{Flow, Held, Outgoing};

use crate::helpers::{agent_finding, rfc3339, uuid7};
use crate::socket::Shared;

const BATCH: usize = 500;

struct Line {
    reporter: Box<dyn Reporter>,
    buffer: Mutex<Outgoing>,
}

pub struct Delivery {
    host: Host,
    lines: Vec<Line>,
    shared: Shared,
}

impl Delivery {
    pub fn new(
        host: Host,
        reporters: Vec<Box<dyn Reporter>>,
        buffers: Vec<Outgoing>,
        shared: Shared,
    ) -> Self {
        let lines = reporters
            .into_iter()
            .zip(buffers)
            .map(|(reporter, buffer)| Line {
                reporter,
                buffer: Mutex::new(buffer),
            })
            .collect();

        Delivery {
            host,
            lines,
            shared,
        }
    }

    pub fn replay(&self) -> Vec<Finding> {
        let mut raised = Vec::new();

        for line in &self.lines {
            let waiting = self.with_buffer(line, |buffer| buffer.held());
            if waiting.records.held > 0 {
                eprintln!(
                    "  outgoing: {} finding(s) waiting for {} since the last run, oldest {}",
                    waiting.records.held,
                    line.reporter.name(),
                    waiting.oldest_at.as_deref().unwrap_or("unknown"),
                );
            }
            raised.extend(self.hand_to(line, &[]));
        }

        raised
    }

    pub fn send(&self, findings: &[Finding]) -> Vec<Finding> {
        let mut raised = Vec::new();

        for line in &self.lines {
            raised.extend(self.hand_to(line, findings));
        }

        raised
    }

    pub fn buffers(&self) -> Vec<BufferStatus> {
        self.lines
            .iter()
            .map(|line| {
                self.with_buffer(line, |buffer| {
                    let held = buffer.held();
                    BufferStatus {
                        receiver: buffer.name().to_string(),
                        pending: held.records.held,
                        pending_ceiling: held.records.ceiling,
                        bytes: held.bytes.held,
                        bytes_ceiling: held.bytes.ceiling,
                        dropped_total: held.dropped,
                        oldest_at: held.oldest_at,
                    }
                })
            })
            .collect()
    }

    pub fn damaged(&self) -> Vec<(String, usize)> {
        self.lines
            .iter()
            .map(|line| {
                self.with_buffer(line, |buffer| (buffer.name().to_string(), buffer.damaged()))
            })
            .filter(|(_, damaged)| *damaged > 0)
            .collect()
    }

    fn hand_to(&self, line: &Line, findings: &[Finding]) -> Vec<Finding> {
        let name = line.reporter.name().to_string();
        let wanted: Vec<Finding> = findings
            .iter()
            .filter(|finding| line.reporter.wants(finding))
            .cloned()
            .collect();

        let mut raised = Vec::new();
        let (kept, mut pending, held) = self.with_buffer(line, |buffer| {
            let kept = buffer.keep(&wanted);
            (kept, buffer.waiting(), buffer.held())
        });
        let from_the_buffer = pending.len();

        match kept {
            Ok(flow) => raised.extend(said(&name, flow, &held)),
            Err(error) => {
                eprintln!(
                    "{} outgoing buffer for {name} not written: {error}. What was found goes out \
                     unbuffered, and a delivery that fails now leaves it in the local history alone",
                    rfc3339::now()
                );
                pending.extend(wanted.iter().cloned());
            }
        }
        if pending.is_empty() {
            return raised;
        }

        let (accepted, failure) = self.push(line, &pending);
        let (drained, held) = self.with_buffer(line, |buffer| {
            let drained = match buffer.delivered(accepted.min(from_the_buffer)) {
                Ok(flow) => flow,
                Err(error) => {
                    eprintln!(
                        "{} outgoing buffer for {name} not shortened: {error}",
                        rfc3339::now()
                    );
                    Flow::Steady
                }
            };
            (drained, buffer.held())
        });
        raised.extend(said(&name, drained, &held));

        self.shared
            .with(|state| state.record_delivery(&name, rfc3339::now(), failure));
        raised
    }

    fn push(&self, line: &Line, pending: &[Finding]) -> (usize, Option<String>) {
        let mut accepted = 0;
        let mut refused = None;

        for chunk in pending.chunks(BATCH) {
            match line.reporter.send(&self.envelope(chunk)) {
                Ok(_) => accepted += chunk.len(),
                Err(ReportError::Malformed(what)) => {
                    eprintln!(
                        "{} reporter {} cannot be given {} finding(s) by any retry: {what}",
                        rfc3339::now(),
                        line.reporter.name(),
                        chunk.len()
                    );
                    accepted += chunk.len();
                    refused = Some(format!("malformed document: {what}"));
                }
                Err(error) => {
                    eprintln!(
                        "{} reporter {} failed: {error}",
                        rfc3339::now(),
                        line.reporter.name()
                    );
                    return (accepted, Some(error.to_string()));
                }
            }
        }

        (accepted, refused)
    }

    fn envelope(&self, findings: &[Finding]) -> Envelope {
        Envelope {
            schema_version: SCHEMA_VERSION,
            batch_id: uuid7::mint(),
            sent_at: rfc3339::now(),
            producer: Producer {
                name: "vigil".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            host: self.host.clone(),
            findings: findings.to_vec(),
        }
    }

    fn with_buffer<R>(&self, line: &Line, act: impl FnOnce(&mut Outgoing) -> R) -> R {
        let mut buffer = line
            .buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        act(&mut buffer)
    }
}

fn said(reporter: &str, flow: Flow, held: &Held) -> Vec<Finding> {
    match flow {
        Flow::Steady => Vec::new(),
        Flow::Dropping { dropped, .. } => {
            vec![agent_finding::buffer_dropping(reporter, dropped, held)]
        }
        Flow::Drained { dropped } => vec![agent_finding::buffer_drained(reporter, dropped)],
    }
}
