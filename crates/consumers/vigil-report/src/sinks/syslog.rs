use std::io::Write;
use std::os::unix::net::{UnixDatagram, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use vigil_model::Envelope;

use crate::formats::syslog::{Rfc5424, SyslogFacility};
use crate::{Delivery, ReportError, Reporter};

pub const LOCAL_SOCKET: &str = "/dev/log";

pub struct SyslogSink {
    name: String,
    path: PathBuf,
    format: Rfc5424,
    link: Mutex<Option<Link>>,
}

enum Link {
    Datagram(UnixDatagram),
    Stream(UnixStream),
}

impl SyslogSink {
    pub fn open(
        name: impl Into<String>,
        path: impl AsRef<Path>,
        facility: SyslogFacility,
        app_name: impl Into<String>,
    ) -> Result<Self, String> {
        let path = path.as_ref().to_path_buf();
        let link = connect(&path)?;

        Ok(SyslogSink {
            name: name.into(),
            path,
            format: Rfc5424::new(facility, app_name, std::process::id()),
            link: Mutex::new(Some(link)),
        })
    }
}

impl Reporter for SyslogSink {
    fn name(&self) -> &str {
        &self.name
    }

    fn send(&self, envelope: &Envelope) -> Result<Delivery, ReportError> {
        let mut link = self
            .link
            .lock()
            .map_err(|_| ReportError::Transient("syslog socket poisoned by a panic".into()))?;

        let mut accepted = 0u64;
        let mut lost = 0u64;
        let mut first_failure: Option<String> = None;

        for finding in &envelope.findings {
            let line = self.format.line(finding, &envelope.host, &envelope.sent_at);

            if first_failure.is_some() && link.is_none() {
                lost += 1;
                continue;
            }

            match emit(&mut link, &self.path, &line) {
                Ok(()) => accepted += 1,
                Err(error) => {
                    lost += 1;
                    first_failure.get_or_insert(error);
                }
            }
        }

        match first_failure {
            None => Ok(Delivery::Accepted {
                accepted,
                duplicates: 0,
            }),
            Some(error) => Err(ReportError::Transient(format!(
                "{accepted} record(s) reached {}, {lost} did not: {error}",
                self.path.display()
            ))),
        }
    }
}

fn emit(link: &mut Option<Link>, path: &Path, line: &str) -> Result<(), String> {
    for attempt in 0..2 {
        if link.is_none() {
            *link = Some(connect(path)?);
        }

        let Some(open) = link.as_mut() else {
            continue;
        };
        match write(open, line) {
            Ok(()) => return Ok(()),
            Err(error) => {
                *link = None;
                if attempt == 1 {
                    return Err(error.to_string());
                }
            }
        }
    }

    Err("syslog socket could not be written".to_string())
}

fn connect(path: &Path) -> Result<Link, String> {
    let datagram = UnixDatagram::unbound().and_then(|socket| {
        socket.connect(path)?;
        Ok(socket)
    });
    match datagram {
        Ok(socket) => Ok(Link::Datagram(socket)),
        Err(as_datagram) => match UnixStream::connect(path) {
            Ok(stream) => Ok(Link::Stream(stream)),
            Err(as_stream) => Err(format!(
                "{}: not a datagram socket ({as_datagram}) and not a stream socket ({as_stream})",
                path.display()
            )),
        },
    }
}

fn write(link: &mut Link, line: &str) -> std::io::Result<()> {
    match link {
        Link::Datagram(socket) => socket.send(line.as_bytes()).map(|_| ()),
        Link::Stream(stream) => {
            let mut framed = String::with_capacity(line.len() + 1);
            framed.push_str(line);
            framed.push('\n');
            stream.write_all(framed.as_bytes())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::atomic::{AtomicU32, Ordering};

    use serde_json::json;
    use vigil_model::{
        Evidence, Finding, Host, Kind, KnownKind, Os, Producer, SCHEMA_VERSION, Severity, State,
        Subject,
    };

    use super::*;

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
}
