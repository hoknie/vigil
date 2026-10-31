use vigil_collect::Health;

use super::ResourcesCollector;
use super::filesystems::filesystems;
use super::source::{
    BOOT_SESSION, BOOT_TIME, MEMORY_LEVEL, MEMORY_SIZE, PROCESSOR_LOAD, boot_session, memory,
    processors,
};

const WHICH_BOOT: &str = "which boot this host is running is unknown, so a restart of it cannot be told from a host \
     that has been up all along";

const WHEN_IT_BOOTED: &str = "the moment this host booted is unknown, so a clock stepping away from the one the rest of \
     the fleet keeps will pass unseen";

const HOW_BIG: &str = "the memory this host has is unknown";

const HOW_BUSY: &str = "how busy the processors of this host are will not be seen";

impl ResourcesCollector {
    pub(super) fn health(&self) -> Health {
        let values = [
            (boot_session().is_none(), BOOT_SESSION, WHICH_BOOT),
            ((self.booted)().is_none(), BOOT_TIME, WHEN_IT_BOOTED),
            (memory().is_none(), MEMORY_SIZE, HOW_BIG),
            (processors().is_none(), PROCESSOR_LOAD, HOW_BUSY),
        ];
        let read_nothing = values.iter().all(|(missing, ..)| *missing);

        let mut complaints: Vec<String> = values
            .into_iter()
            .filter(|(missing, ..)| *missing)
            .map(|(_, value, lost)| format!("{value} could not be read from the kernel, so {lost}"))
            .collect();
        if memory().is_some_and(|memory| memory.available_percent_step.is_none()) {
            complaints.push(format!(
                "{MEMORY_LEVEL} could not be read from the kernel, so memory running out will not be seen"
            ));
        }
        let (measured, unmeasured) = filesystems();
        complaints.extend(unmeasured);

        match (complaints.is_empty(), read_nothing && measured.is_empty()) {
            (true, _) => Health::Ok,
            (false, true) => Health::Unavailable(complaints.join("; ")),
            (false, false) => Health::Degraded(complaints.join("; ")),
        }
    }
}
