use std::fs::OpenOptions;
use std::path::Path;

use vigil_report::{NdjsonSink, Reporter, SyslogFacility, SyslogSink};

use crate::Receiver;

pub fn build(receivers: &[Receiver]) -> Result<Vec<Box<dyn Reporter>>, String> {
    let mut reporters: Vec<Box<dyn Reporter>> = Vec::new();

    for receiver in receivers {
        match receiver {
            Receiver::Ndjson { path } => {
                if let Some(parent) = Path::new(path).parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|error| format!("{}: {error}", parent.display()))?;
                }
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|error| format!("the ndjson reporter cannot open {path}: {error}"))?;
                reporters.push(Box::new(NdjsonSink::new("ndjson", file)));
            }
            Receiver::Syslog { facility } => {
                let facility = SyslogFacility::parse(facility)
                    .map_err(|error| format!("the syslog reporter: {error}"))?;
                reporters.push(Box::new(
                    SyslogSink::open("syslog", vigil_report::LOCAL_SOCKET, facility, "vigil")
                        .map_err(|error| format!("the syslog reporter cannot connect: {error}"))?,
                ));
            }
            Receiver::Webhook { .. } => {
                return Err(
                    "the webhook reporter is configured but not implemented in this build".into(),
                );
            }
            Receiver::HostFindings { .. } => {
                return Err(
                    "the host-findings reporter is configured but not implemented in this build"
                        .into(),
                );
            }
        }
    }

    Ok(reporters)
}

pub fn names(reporters: &[Box<dyn Reporter>]) -> Vec<String> {
    reporters
        .iter()
        .map(|reporter| reporter.name().to_string())
        .collect()
}
