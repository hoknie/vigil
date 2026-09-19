use std::fs::File;
use std::io::{ErrorKind, Read};

use vigil_collect::Health;

use super::LaunchesCollector;
use crate::spool::{ABSENT, CEILING_BYTES, ESLOGGER, REFUSED, SpoolerStatus, dropped_note};

const JOB: &str = "vigil.launches";

const SPOOLER: &str = "/usr/local/libexec/vigil/vigil-launches-spool";

const LOG: &str = "/usr/local/var/log/vigil/vigil-launches.log";

fn what_eslogger_needs() -> String {
    format!(
        "{ESLOGGER} runs as root and needs Full Disk Access, given to {SPOOLER} in System \
         Settings, Privacy & Security, Full Disk Access"
    )
}

impl LaunchesCollector {
    pub(super) fn health(&self) -> Health {
        let status = match SpoolerStatus::read(&self.status_path) {
            Ok(status) => status,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Health::Unavailable(format!(
                    "program launches are not visible: the launchd job {JOB} has never run on \
                     this Mac, and there is no {}. `vigild collector launches enable` starts \
                     it; {}",
                    self.status_path.display(),
                    what_eslogger_needs()
                ));
            }
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                return Health::Unavailable(format!(
                    "program launches are not visible to this agent: {} cannot be read. The \
                     job writes it 0600 as root; run as root",
                    self.status_path.display()
                ));
            }
            Err(error) => {
                return Health::Degraded(format!(
                    "{} cannot be read ({error}), so whether eslogger is delivering is \
                     unknown. {LOG} says what the job did",
                    self.status_path.display()
                ));
            }
        };

        let why = status.why.clone().unwrap_or_default();
        match status.state.as_str() {
            ABSENT => {
                return Health::Unavailable(format!(
                    "program launches are not visible: {ESLOGGER} is not on this Mac. It \
                     ships with macOS 13 and newer, and the job said: {why}"
                ));
            }
            REFUSED => {
                return Health::Unavailable(format!(
                    "program launches are not visible: eslogger refused to start at {}: \
                     {why}. {}; launchd tries again every 30 seconds",
                    status.since,
                    what_eslogger_needs()
                ));
            }
            _ if !status.is_running() => {
                return Health::Degraded(format!(
                    "eslogger stopped at {}: {why}. Launches after that moment are not \
                     visible until launchd starts the job {JOB} again; {LOG} says why it \
                     stopped",
                    status.since
                ));
            }
            _ => {}
        }

        if let Some(note) = self.dropped() {
            return Health::Degraded(format!(
                "the job {JOB} {note}. Launches from that window are not visible; later ones \
                 are. Usual cause: this daemon was stopped while eslogger kept delivering. \
                 The spool at {} holds {CEILING_BYTES} bytes and drops the oldest first",
                self.spool_path.display()
            ));
        }

        if self.keep_arguments && !status.arguments_recorded {
            return Health::Degraded(format!(
                "record_arguments is on in the configuration and the job {JOB} spools \
                 launches without their arguments: it read the configuration before it was \
                 changed. `launchctl kickstart -k system/{JOB}` makes it read it again"
            ));
        }

        Health::Ok
    }

    fn dropped(&self) -> Option<String> {
        let mut file = File::open(&self.spool_path).ok()?;
        let mut head = [0u8; 512];
        let read = file.read(&mut head).ok()?;
        let line = head[..read]
            .split(|byte| *byte == b'\n')
            .next()
            .unwrap_or_default();
        dropped_note(line)
    }
}
