use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};

use crate::{Delivery, Reporter, SyslogFacility, SyslogSink};
use vigil_model::Envelope;

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};

use serde_json::json;
use vigil_model::{
    Evidence, Finding, Host, Kind, KnownKind, Os, Producer, SCHEMA_VERSION, Severity, State,
    Subject,
};

fn socket_path(tag: &str) -> PathBuf {
    static SEQUENCE: AtomicU32 = AtomicU32::new(0);
    let unique = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("vigil-{tag}-{}-{unique}.sock", std::process::id()))
}

fn envelope(findings: Vec<Finding>) -> Envelope {
    Envelope {
        schema_version: SCHEMA_VERSION,
        batch_id: "0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e2".into(),
        sent_at: "2026-09-09T12:04:05.000Z".into(),
        producer: Producer {
            name: "vigil".into(),
            version: "0.1.0".into(),
        },
        host: Host {
            host_id: "b7f1c0a2".into(),
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
            tags: BTreeMap::new(),
            peer: None,
        },
        findings,
    }
}

fn finding(key: &str) -> Finding {
    Finding {
        event_id: "0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e1".into(),
        finding_key: key.to_string(),
        kind: Kind::Known(KnownKind::PortListenNew),
        severity: Severity::High,
        state: State::Open,
        observed_at: "2026-09-09T12:04:02.311Z".into(),
        first_seen_at: "2026-09-09T12:04:02.311Z".into(),
        occurrences: 1,
        title: format!("New listening socket {key}"),
        subject: Subject {
            object: "socket".into(),
            key: json!({"protocol": "tcp"}),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "path".into(),
            value: "/tmp/nc".into(),
        }],
        redacted: Vec::new(),
        rule: Some("new_listening_port".into()),
        labels: BTreeMap::new(),
    }
}

fn sink(path: &Path) -> SyslogSink {
    SyslogSink::open(
        "syslog",
        path,
        SyslogFacility::parse("local4").expect("known"),
        "vigil",
    )
    .expect("the socket is listening")
}

fn received(socket: &UnixDatagram) -> String {
    let mut buffer = [0u8; 4096];
    let read = socket.recv(&mut buffer).expect("a datagram");
    String::from_utf8_lossy(&buffer[..read]).to_string()
}

#[test]
fn a_socket_that_is_not_there_is_refused_by_name_at_start_up() {
    let path = socket_path("absent");

    let error = SyslogSink::open(
        "syslog",
        &path,
        SyslogFacility::parse("daemon").expect("known"),
        "vigil",
    )
    .map(|_| ())
    .expect_err("an agent that reports to nothing must not start quietly");

    assert!(
        error.contains(path.to_str().expect("utf-8")),
        "the path belongs in the message: {error}"
    );
}

#[test]
fn every_finding_of_a_batch_becomes_its_own_record() {
    let path = socket_path("batch");
    let listener = UnixDatagram::bind(&path).expect("bind");
    let sink = sink(&path);

    let delivery = sink
        .send(&envelope(vec![
            finding("tcp|0.0.0.0:4444"),
            finding("tcp|0.0.0.0:4445"),
        ]))
        .expect("both land");

    assert_eq!(
        delivery,
        Delivery::Accepted {
            accepted: 2,
            duplicates: 0
        }
    );
    let first = received(&listener);
    let second = received(&listener);
    assert!(first.contains("0.0.0.0:4444"), "{first}");
    assert!(second.contains("0.0.0.0:4445"), "{second}");
    assert!(first.starts_with("<163>1 "), "local4 + err: {first}");

    drop(listener);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_syslog_restarted_underneath_the_daemon_starts_receiving_again_by_itself() {
    let path = socket_path("restart");
    let listener = UnixDatagram::bind(&path).expect("bind");
    let sink = sink(&path);

    sink.send(&envelope(vec![finding("tcp|0.0.0.0:1")]))
        .expect("the first one lands");
    assert!(received(&listener).contains("0.0.0.0:1"));

    drop(listener);
    std::fs::remove_file(&path).expect("unlink");
    let failure = sink
        .send(&envelope(vec![finding("tcp|0.0.0.0:2")]))
        .expect_err("with nothing listening this must be a failure, not a silent success");
    assert!(
        failure.to_string().contains("0 record(s) reached"),
        "and it must say how much was lost: {failure}"
    );

    let listener = UnixDatagram::bind(&path).expect("rebind");
    sink.send(&envelope(vec![finding("tcp|0.0.0.0:3")]))
        .expect("delivery resumes without restarting the daemon");
    assert!(received(&listener).contains("0.0.0.0:3"));

    drop(listener);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_stream_socket_is_spoken_to_as_well_as_a_datagram_one() {
    use std::io::Read;
    use std::os::unix::net::UnixListener;

    let path = socket_path("stream");
    let listener = UnixListener::bind(&path).expect("bind");
    let sink = sink(&path);
    let accepted = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut text = String::new();
        let _ = stream.read_to_string(&mut text);
        text
    });

    sink.send(&envelope(vec![finding("tcp|0.0.0.0:9")]))
        .expect("a stream socket is a syslog socket too");
    drop(sink);

    let text = accepted.join().expect("the reader thread");
    assert!(text.contains("0.0.0.0:9"), "{text}");
    assert!(
        text.ends_with('\n'),
        "a stream has no frames, so the record needs a terminator: {text}"
    );

    let _ = std::fs::remove_file(&path);
}

#[test]
fn an_empty_batch_is_a_delivery_of_nothing_rather_than_an_empty_record() {
    let path = socket_path("empty");
    let listener = UnixDatagram::bind(&path).expect("bind");
    let sink = sink(&path);

    assert_eq!(
        sink.send(&envelope(Vec::new())).expect("nothing to do"),
        Delivery::Accepted {
            accepted: 0,
            duplicates: 0
        }
    );

    drop(listener);
    let _ = std::fs::remove_file(&path);
}
