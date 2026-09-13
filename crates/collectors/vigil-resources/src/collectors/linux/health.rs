use super::ResourcesCollector;
use super::files::{BOOT_ID, Files, MEMINFO, Refusal, UPTIME};
use vigil_collect::Health;

const WHICH_BOOT: &str = "which boot this host is running is unknown, so a restart of it cannot be told from a host \
     that has been up all along";

const WHEN_IT_BOOTED: &str = "the moment this host booted is unknown, so a clock stepping away from the one the rest of \
     the fleet keeps will pass unseen";

const HOW_BIG: &str = "the memory and the swap this host has are unknown";

impl ResourcesCollector {
    pub(super) fn health(&self) -> Health {
        let files = [
            (self.files.boot_id().err(), BOOT_ID, WHICH_BOOT),
            (self.files.uptime_seconds().err(), UPTIME, WHEN_IT_BOOTED),
            (self.files.memory().err(), MEMINFO, HOW_BIG),
        ];
        let read_nothing = files.iter().all(|(refusal, ..)| refusal.is_some());

        let mut complaints: Vec<String> = files
            .into_iter()
            .filter_map(|(refusal, file, lost)| {
                refusal.map(|refusal| said(&self.files, refusal, file, lost))
            })
            .collect();
        let (filesystems, unmeasured) = self.filesystems();
        complaints.extend(unmeasured);

        if complaints.is_empty() {
            return Health::Ok;
        }

        match read_nothing && filesystems.is_empty() {
            true => Health::Unavailable(format!(
                "{}. A /proc that is not mounted, or one a namespace shows a slice of, is a \
                 reading this agent cannot take rather than a host with nothing on it",
                complaints.join("; ")
            )),
            false => Health::Degraded(complaints.join("; ")),
        }
    }
}

fn said(files: &Files, refusal: Refusal, file: &str, lost: &str) -> String {
    let shown = files.path(file).display().to_string();

    match refusal {
        Refusal::Absent => format!("{shown} is not there, so {lost}"),
        Refusal::Denied => format!("{shown} cannot be read by this agent, so {lost}"),
        Refusal::Unreadable => format!(
            "{shown} is in a shape this build does not know, so {lost}: the reading is refused \
             rather than half read"
        ),
    }
}
