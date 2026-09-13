use vigil_model::Snapshot;

use super::kind::Kind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum List {
    #[default]
    Units,
    Timers,
    Cron,
    Modules,
    Files,
    Other,
}

impl List {
    pub const ALL: &'static [List] = &[
        List::Units,
        List::Timers,
        List::Cron,
        List::Modules,
        List::Files,
        List::Other,
    ];

    pub fn shown(self, reading: &Snapshot) -> bool {
        self != List::Other || holds_something_unknown(reading)
    }

    pub fn name(self) -> &'static str {
        match self {
            List::Units => "units",
            List::Timers => "timers",
            List::Cron => "cron",
            List::Modules => "modules",
            List::Files => "files",
            List::Other => "other",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            List::Units => "UNITS",
            List::Timers => "TIMERS",
            List::Cron => "CRON",
            List::Modules => "MODULES",
            List::Files => "FILES",
            List::Other => "OTHER",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            List::Units => "THE SELECTED UNIT",
            List::Timers => "THE SELECTED TIMER",
            List::Cron => "THE SELECTED CRON JOB",
            List::Modules => "THE SELECTED MODULE",
            List::Files => "THE SELECTED FILE",
            List::Other => "THE SELECTED OBJECT",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            List::Units => "every systemd unit found on disk, and the command it runs",
            List::Timers => "every systemd timer, when it fires and what it starts",
            List::Cron => "every cron job, and the whole command it runs",
            List::Modules => "the kernel modules loaded right now",
            List::Files => {
                "files whose contents other processes execute: shell profiles, rc.local and \
                 /etc/ld.so.preload"
            }
            List::Other => "objects of a kind this console does not know, from a newer agent",
        }
    }

    pub fn thing(self) -> &'static str {
        match self {
            List::Units => "unit",
            List::Timers => "timer",
            List::Cron => "cron job",
            List::Modules => "kernel module",
            List::Files => "file",
            List::Other => "object of an unknown kind",
        }
    }

    pub fn things(self, how_many: usize) -> String {
        if how_many == 1 {
            return self.thing().to_string();
        }
        match self {
            List::Units => "units".to_string(),
            List::Timers => "timers".to_string(),
            List::Cron => "cron job(s)".to_string(),
            List::Modules => "kernel modules".to_string(),
            List::Files => "files".to_string(),
            List::Other => "objects of a kind this console does not know".to_string(),
        }
    }

    pub fn empty(self) -> &'static str {
        match self {
            List::Units => "This host starts nothing from a systemd unit.",
            List::Timers => "No systemd timer is set on this host.",
            List::Cron => "No cron job was found, in any of the four places cron reads.",
            List::Modules => "No kernel module is loaded.",
            List::Files => {
                "None of the files other processes execute was found: no shell profile, no \
                 rc.local, no /etc/ld.so.preload."
            }
            List::Other => "Nothing of a kind this console does not know.",
        }
    }

    pub fn holding(key: &str) -> List {
        match Kind::of(key) {
            Kind::Unit => List::Units,
            Kind::Timer => List::Timers,
            Kind::Cron => List::Cron,
            Kind::Module | Kind::ModulesUnreadable => List::Modules,
            Kind::Script | Kind::Preload => List::Files,
            Kind::Unknown => List::Other,
        }
    }
}

fn holds_something_unknown(reading: &Snapshot) -> bool {
    reading
        .items
        .keys()
        .any(|key| Kind::of(key) == Kind::Unknown)
}
