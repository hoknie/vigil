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
