use vigil_collect::{CollectError, Health};

use super::collector::FirewallCollector;
use crate::parsers::{Understood, understood};
use crate::types::{MACOS_JOB, MACOS_WRITER};

const LOG: &str = "/usr/local/var/log/vigil/vigil-firewall.log";

fn how_it_is_written() -> String {
    format!(
        "this reading is written by the launchd job {MACOS_JOB}, which runs {MACOS_WRITER} as \
         root: it asks /sbin/pfctl and socketfilterfw and nothing else, and the agent never \
         starts a program of its own"
    )
}

impl FirewallCollector {
    pub(super) fn health(&self) -> Health {
        let (dump, age) = match self.dump() {
            Ok(read) => read,
            Err(refusal) => return self.refused(&refusal),
        };

        let mut said: Vec<String> = Vec::new();
        if let Some(age) = age {
            said.push(format!(
                "the reading in {} was written {age} seconds ago, more than the {} this \
                 collector allows: what it says about this Mac may have been true and no \
                 longer is. {}; a job launchd is not starting is the usual cause \
                 (launchctl print system/{MACOS_JOB})",
                self.dump_path.display(),
                self.stale_after_seconds(),
                how_it_is_written()
            ));
        }

        let Understood {
            pf,
            application_firewall,
        } = understood(&dump);
        match &pf {
            Err(why) => said.push(format!(
                "pf could not be read ({why}), so what pf filters on this Mac is unknown — \
                 which is not the same as pf filtering nothing. pfctl answers root alone, and \
                 the job must run as root"
            )),
            Ok(read) if !read.anchors_unread.is_empty() => said.push(format!(
                "{} anchor(s) of pf could not be read: {}",
                read.anchors_unread.len(),
                read.anchors_unread.join("; ")
            )),
            Ok(_) => {}
        }
        if let Err(why) = &application_firewall {
            said.push(format!(
                "the Application Firewall could not be read ({why}), so which programs it lets \
                 in is unknown"
            ));
        }

        match said.is_empty() {
            true => Health::Ok,
            false => Health::Degraded(said.join(" · ")),
        }
    }

    fn refused(&self, refusal: &CollectError) -> Health {
        let shown = self.dump_path.display();
        match refusal {
            CollectError::Absent(_) => Health::Unavailable(format!(
                "{shown} is not there, so what this Mac filters is unknown — which is not the \
                 same as nothing. {}. Either the job has never run (vigild collector firewall \
                 enable), or it was switched off",
                how_it_is_written()
            )),
            CollectError::Denied(_) => Health::Unavailable(format!(
                "{shown} cannot be read by this agent, so what this Mac filters is unknown. The \
                 directory is 0700 root:wheel and so is the file; run as root"
            )),
            other => Health::Degraded(format!(
                "{other}. {}; its last run left this behind. {LOG} and `launchctl print \
                 system/{MACOS_JOB}` say why",
                how_it_is_written()
            )),
        }
    }
}
