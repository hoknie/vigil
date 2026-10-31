use std::sync::Mutex as StandardMutex;
use std::sync::atomic::{AtomicBool, Ordering};

use vigil_model::{Envelope, Finding, Kind, KnownKind};
use vigil_report::Delivery as Accepted;
use vigil_report::{ReportError, Reporter};
use vigil_store::{Limits, Outgoing};

use super::Delivery;
use super::delivery::holdable;
use crate::socket::{Shared, fixture};

struct Scripted {
    name: String,
    refusing: AtomicBool,
    malformed: AtomicBool,
    taken: StandardMutex<Vec<Finding>>,
    attempts: StandardMutex<Vec<(String, Vec<String>)>>,
    documents: StandardMutex<Vec<serde_json::Value>>,
    only: Option<&'static str>,
}

impl Scripted {
    fn new(name: &str) -> Self {
        Scripted {
            name: name.to_string(),
            refusing: AtomicBool::new(false),
            malformed: AtomicBool::new(false),
            taken: StandardMutex::new(Vec::new()),
            attempts: StandardMutex::new(Vec::new()),
            documents: StandardMutex::new(Vec::new()),
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

    fn attempts(&self) -> Vec<(String, Vec<String>)> {
        self.attempts.lock().expect("not poisoned").clone()
    }
}

impl Reporter for Scripted {
    fn name(&self) -> &str {
        &self.name
    }

    fn send(&self, envelope: &Envelope) -> Result<vigil_report::Delivery, ReportError> {
        self.documents
            .lock()
            .expect("not poisoned")
            .push(serde_json::to_value(envelope).expect("plain data"));
        self.attempts.lock().expect("not poisoned").push((
            envelope.batch_id.clone(),
            envelope
                .findings
                .iter()
                .map(|finding| finding.finding_key.clone())
                .collect(),
        ));
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
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "vigild-delivery-{}-{name}-{}.ndjson",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_nanos()
                + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
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

    let mut other = found("agent.collector|network");
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

fn batch_file(path: &std::path::Path) -> std::path::PathBuf {
    path.with_extension("batch")
}

fn named(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| key.to_string()).collect()
}

#[test]
fn a_retry_carries_the_same_batch_id_and_the_same_findings() {
    let path = temporary_path("same-batch");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    receiver.refusing.store(true, Ordering::Relaxed);
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

    delivery.send(&[found("a"), found("b"), found("c")]);
    delivery.send(&[found("d")]);

    let attempts = receiver.attempts();
    assert_eq!(attempts.len(), 2, "{attempts:?}");
    assert_eq!(
        attempts[1], attempts[0],
        "a receiver tells a retry from a new batch by its id, and only if the id stays"
    );
    assert_eq!(attempts[1].1, named(&["a", "b", "c"]));
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(batch_file(&path));
}

#[test]
fn findings_found_after_a_refusal_go_in_a_batch_of_their_own() {
    let path = temporary_path("own-batch");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    receiver.refusing.store(true, Ordering::Relaxed);
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

    delivery.send(&[found("a"), found("b"), found("c")]);
    receiver.refusing.store(false, Ordering::Relaxed);
    delivery.send(&[found("d"), found("e")]);

    let attempts = receiver.attempts();
    assert_eq!(attempts.len(), 3, "{attempts:?}");
    assert_eq!(attempts[1], attempts[0]);
    assert_ne!(attempts[2].0, attempts[0].0);
    assert_eq!(
        attempts[2].1,
        named(&["d", "e"]),
        "what came after the refusal is a new batch, not a larger copy of the old one"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_retry_after_a_restart_carries_the_batch_id_from_before_it() {
    let path = temporary_path("restart-batch");
    let before = {
        let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
        receiver.refusing.store(true, Ordering::Relaxed);
        let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));
        delivery.send(&[found("a"), found("b")]);
        receiver.attempts()
    };

    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));
    delivery.replay();

    assert_eq!(
        receiver.attempts(),
        before,
        "a restart while the receiver is down is the retry that matters most"
    );
    assert!(
        !batch_file(&path).exists(),
        "and once taken it is forgotten"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_batch_holding_findings_the_buffer_never_kept_is_not_held() {
    assert!(holdable(0, 3, 3), "a batch wholly from the buffer is held");
    assert!(holdable(2, 3, 5));
    assert!(
        !holdable(0, 3, 2),
        "a finding the buffer never kept cannot be sent again, so the batch cannot be either"
    );
    assert!(!holdable(2, 3, 4));
    assert!(!holdable(0, 0, 4), "and an empty batch is not a batch");
}

#[test]
fn a_batch_no_retry_can_fix_is_not_held() {
    let path = temporary_path("malformed-batch");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    receiver.refusing.store(true, Ordering::Relaxed);
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

    delivery.send(&[found("a")]);
    assert!(batch_file(&path).exists(), "a refused batch is held");
    receiver.malformed.store(true, Ordering::Relaxed);
    delivery.send(&[]);
    assert!(
        !batch_file(&path).exists(),
        "a batch that no retry can fix is not waiting for one"
    );
    receiver.malformed.store(false, Ordering::Relaxed);
    receiver.refusing.store(false, Ordering::Relaxed);
    delivery.send(&[found("b")]);

    let attempts = receiver.attempts();
    assert_eq!(attempts.len(), 3, "{attempts:?}");
    assert_ne!(attempts[2].0, attempts[0].0);
    assert_eq!(attempts[2].1, named(&["b"]));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_delivery_accepted_at_once_writes_no_batch_file() {
    let path = temporary_path("no-batch");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()));

    delivery.send(&[found("a"), found("b")]);

    assert_eq!(receiver.keys(), named(&["a", "b"]));
    assert!(
        !batch_file(&path).exists(),
        "the price of a held batch is paid on a refusal, never on the ordinary path"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn one_batch_id_never_goes_out_with_two_different_sets_of_findings() {
    let path = temporary_path("one-id");
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

    delivery.send(&[found("a"), found("b"), found("c")]);
    delivery.send(&[found("d")]);
    delivery.send(&[found("e"), found("f")]);
    receiver.refusing.store(false, Ordering::Relaxed);
    delivery.send(&[found("g")]);

    let mut seen: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for (batch_id, keys) in receiver.attempts() {
        let first = seen.entry(batch_id.clone()).or_insert_with(|| keys.clone());
        assert_eq!(
            *first, keys,
            "batch {batch_id} went out as {first:?} and again as {keys:?}"
        );
    }
    assert!(
        !receiver.keys().contains(&"a".to_string()),
        "the ceiling displaced the oldest, and the batch they headed went with them"
    );
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(batch_file(&path));
}

#[test]
fn a_document_carries_the_addresses_the_host_has_when_it_is_sent() {
    let path = temporary_path("addresses");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = calls.clone();
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()))
        .reading_addresses_with(move || match counted.fetch_add(1, Ordering::Relaxed) {
            0 => vec!["10.0.4.17".to_string()],
            _ => vec!["10.0.4.40".to_string()],
        });

    delivery.send(&[found("a"), found("b")]);
    delivery.send(&[]);
    delivery.send(&[found("c")]);

    let documents = receiver.documents.lock().expect("not poisoned").clone();
    assert_eq!(documents.len(), 2, "{documents:?}");
    assert_eq!(
        documents[0]["host"]["addresses"],
        serde_json::json!(["10.0.4.17"])
    );
    assert_eq!(
        documents[1]["host"]["addresses"],
        serde_json::json!(["10.0.4.40"]),
        "an address the host was given by DHCP since the start is the one the next document carries"
    );
    assert_eq!(
        calls.load(Ordering::Relaxed),
        2,
        "the addresses are read when something is sent, not on a round with nothing to send"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_host_with_no_address_but_loopback_sends_no_addresses_field() {
    let path = temporary_path("no-addresses");
    let receiver = std::sync::Arc::new(Scripted::new("ndjson"));
    let delivery = delivery(receiver.clone(), buffer(&path, Limits::outgoing()))
        .reading_addresses_with(Vec::new);

    delivery.send(&[found("a")]);

    let documents = receiver.documents.lock().expect("not poisoned").clone();
    assert!(
        documents[0]["host"].get("addresses").is_none(),
        "an empty list is no field, not a list saying the host has no address: {}",
        documents[0]["host"]
    );
    let _ = std::fs::remove_file(&path);
}
