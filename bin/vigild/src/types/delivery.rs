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

#[cfg(test)]
mod tests {
    use std::sync::Mutex as StandardMutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use vigil_model::{Kind, KnownKind};
    use vigil_report::Delivery as Accepted;
    use vigil_store::Limits;

    use crate::socket::fixture;

    use super::*;

    struct Scripted {
        name: String,
        refusing: AtomicBool,
        malformed: AtomicBool,
        taken: StandardMutex<Vec<Finding>>,
        only: Option<&'static str>,
    }

    impl Scripted {
        fn new(name: &str) -> Self {
            Scripted {
                name: name.to_string(),
                refusing: AtomicBool::new(false),
                malformed: AtomicBool::new(false),
                taken: StandardMutex::new(Vec::new()),
                only: None,
            }
        }

        fn keys(&self) -> Vec<String> {
            self.taken
                .lock()
                .expect("not poisoned")
                .iter()
                .map(|finding| finding.finding_key.clone())
                .collect()
        }
    }

    impl Reporter for Scripted {
        fn name(&self) -> &str {
            &self.name
        }

        fn send(&self, envelope: &Envelope) -> Result<vigil_report::Delivery, ReportError> {
            if self.malformed.load(Ordering::Relaxed) {
                return Err(ReportError::Malformed("no retry fixes this".into()));
            }
            if self.refusing.load(Ordering::Relaxed) {
                return Err(ReportError::Transient("the receiver is down".into()));
            }
            self.taken
                .lock()
                .expect("not poisoned")
                .extend(envelope.findings.iter().cloned());
            Ok(Accepted::Accepted {
                accepted: envelope.findings.len() as u64,
                duplicates: 0,
            })
        }

        fn wants(&self, finding: &Finding) -> bool {
            match self.only {
                None => true,
                Some(kind) => finding.kind.as_str() == kind,
            }
        }
    }

    fn temporary_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "vigild-delivery-{}-{name}-{}.ndjson",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ))
    }

    fn buffer(path: &std::path::Path, limits: Limits) -> Outgoing {
        Outgoing::open("ndjson", path.to_path_buf(), limits).expect("opens")
    }

    fn found(key: &str) -> Finding {
        let mut finding = fixture::finding(key);
        finding.finding_key = key.to_string();
        finding
    }

    fn delivery(reporter: std::sync::Arc<Scripted>, buffer: Outgoing) -> Delivery {
        struct Handed(std::sync::Arc<Scripted>);

        impl Reporter for Handed {
            fn name(&self) -> &str {
                self.0.name()
            }
            fn send(&self, envelope: &Envelope) -> Result<vigil_report::Delivery, ReportError> {
                self.0.send(envelope)
            }
            fn wants(&self, finding: &Finding) -> bool {
                self.0.wants(finding)
            }
        }

        Delivery::new(
            fixture::host(),
            vec![Box::new(Handed(reporter))],
            vec![buffer],
            Shared::new(fixture::state()),
        )
    }

    fn kinds(findings: &[Finding]) -> Vec<String> {
        findings
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect()
    }

    #[test]
    fn a_finding_a_receiver_would_not_take_is_offered_again_when_it_comes_back() {
        let path = temporary_path("again");
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        receiver.refusing.store(true, Ordering::Relaxed);
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

        delivery.send(&[found("port.listen|tcp|0.0.0.0:4444")]);
        assert!(receiver.keys().is_empty(), "the receiver was down");
        receiver.refusing.store(false, Ordering::Relaxed);
        delivery.send(&[found("port.listen|tcp|0.0.0.0:80")]);

        assert_eq!(
            receiver.keys(),
            vec![
                "port.listen|tcp|0.0.0.0:4444".to_string(),
                "port.listen|tcp|0.0.0.0:80".to_string()
            ],
            "what the receiver missed arrives before what came after it, not instead of it"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn what_was_waiting_when_the_agent_stopped_goes_out_before_anything_new() {
        let path = temporary_path("restart");
        {
            let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
            receiver.refusing.store(true, Ordering::Relaxed);
            let delivery = delivery(receiver, buffer(&path, Limits::outgoing()));
            delivery.send(&[found("port.listen|tcp|0.0.0.0:4444")]);
        }

        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));
        let raised = delivery.replay();

        assert_eq!(receiver.keys(), vec!["port.listen|tcp|0.0.0.0:4444"]);
        assert!(
            raised.is_empty(),
            "a buffer that never dropped says nothing"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_finding_the_receiver_took_is_not_offered_a_second_time() {
        let path = temporary_path("once");
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

        delivery.send(&[found("port.listen|tcp|0.0.0.0:4444")]);
        delivery.send(&[found("port.listen|tcp|0.0.0.0:80")]);

        assert_eq!(
            receiver.keys().len(),
            2,
            "at-least-once is not at-least-once-per-delivery: {:?}",
            receiver.keys()
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_buffer_that_reached_its_ceiling_says_so_in_the_vocabulary_of_the_product() {
        let path = temporary_path("ceiling");
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        receiver.refusing.store(true, Ordering::Relaxed);
        let delivery = delivery(
            receiver,
            buffer(
                &path,
                Limits {
                    findings: 4,
                    journal_bytes: 64 * 1024,
                },
            ),
        );

        for index in 0..4 {
            delivery.send(&[found(&format!("port.listen|tcp|0.0.0.0:{index}"))]);
        }
        let raised = delivery.send(&[found("port.listen|tcp|0.0.0.0:9")]);

        assert_eq!(
            kinds(&raised),
            vec!["agent.buffer.dropping".to_string()],
            "data that was lost must not look like data that was never found"
        );
        assert_eq!(raised[0].finding_key, "agent.buffer|ndjson");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_finding_about_a_full_buffer_is_closed_by_one_about_an_empty_one() {
        let path = temporary_path("drained");
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        receiver.refusing.store(true, Ordering::Relaxed);
        let delivery = delivery(
            receiver.clone(),
            buffer(
                &path,
                Limits {
                    findings: 4,
                    journal_bytes: 64 * 1024,
                },
            ),
        );
        for index in 0..6 {
            delivery.send(&[found(&format!("port.listen|tcp|0.0.0.0:{index}"))]);
        }

        receiver.refusing.store(false, Ordering::Relaxed);
        let raised = delivery.send(&[]);

        assert_eq!(kinds(&raised), vec!["agent.buffer.drained".to_string()]);
        assert_eq!(
            Kind::Known(KnownKind::AgentBufferDrained)
                .as_str()
                .to_string(),
            raised[0].kind.as_str().to_string()
        );
        assert_eq!(
            raised[0].finding_key, "agent.buffer|ndjson",
            "the same object, so one closes the other instead of standing beside it"
        );
        let quiet = delivery.send(&[]);
        assert!(quiet.is_empty(), "and it is said once, not every round");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_document_no_retry_can_fix_does_not_block_the_findings_behind_it() {
        let path = temporary_path("malformed");
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        receiver.malformed.store(true, Ordering::Relaxed);
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

        delivery.send(&[found("port.listen|tcp|0.0.0.0:4444")]);
        receiver.malformed.store(false, Ordering::Relaxed);
        delivery.send(&[found("port.listen|tcp|0.0.0.0:80")]);

        assert_eq!(
            receiver.keys(),
            vec!["port.listen|tcp|0.0.0.0:80".to_string()],
            "a document the receiver can never read is dropped and said out loud, not retried \
             for the life of the host in front of everything else"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_buffer_holds_only_what_its_own_receiver_asked_for() {
        let path = temporary_path("wants");
        let mut receiver = Scripted::new("ndjson");
        receiver.only = Some("port.listen.new");
        receiver.refusing.store(true, Ordering::Relaxed);
        let receiver = std::sync::Arc::new(receiver);
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

        let mut other = found("agent.collector|ports");
        other.kind = Kind::Known(KnownKind::AgentCollectorDegraded);
        delivery.send(&[found("port.listen|tcp|0.0.0.0:4444"), other]);

        let lines = std::fs::read_to_string(&path).expect("readable");
        assert_eq!(
            lines.lines().count(),
            1,
            "a receiver that filters is a receiver that never sees what it filtered: {lines}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
