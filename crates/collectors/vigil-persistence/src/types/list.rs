use std::ops::Bound;

use vigil_model::Snapshot;

use super::kind::Kind;

const LAUNCHD_ROWS: &str = "launchd|";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum List {
    Launchd,
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
        List::Launchd,
        List::Other,
    ];

    pub fn shown(self, reading: &Snapshot) -> bool {
        match self {
            List::Other => holds_something_unknown(reading),
            List::Launchd => holds_launchd_jobs(reading),
            List::Units | List::Timers | List::Modules => !holds_launchd_jobs(reading),
            List::Cron | List::Files => true,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            List::Launchd => "launchd",
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
            List::Launchd => "LAUNCHD",
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
            List::Launchd => "THE SELECTED JOB",
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
            List::Launchd => {
                "every launchd daemon and agent on disk — this Mac's, the system's and each \
                 person's — and the program it starts"
            }
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
            List::Launchd => "launchd job",
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
            List::Launchd => "launchd jobs".to_string(),
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
            List::Launchd => "This Mac starts nothing from a launchd job, which no Mac does.",
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
            Kind::Launchd => List::Launchd,
            Kind::Unit => List::Units,
            Kind::Timer => List::Timers,
            Kind::Cron => List::Cron,
            Kind::Module | Kind::ModulesUnreadable => List::Modules,
            Kind::Script | Kind::Preload => List::Files,
            Kind::Unknown => List::Other,
        }
    }
}

fn holds_launchd_jobs(reading: &Snapshot) -> bool {
    reading
        .items
        .range::<str, _>((Bound::Included(LAUNCHD_ROWS), Bound::Unbounded))
        .next()
        .is_some_and(|(key, _)| key.starts_with(LAUNCHD_ROWS))
}

fn holds_something_unknown(reading: &Snapshot) -> bool {
    let mut spans: Vec<(String, String)> = Kind::NAMED
        .iter()
        .flat_map(|name| {
            [
                (name.to_string(), format!("{name}\u{0}")),
                (format!("{name}|"), format!("{name}}}")),
            ]
        })
        .collect();
    spans.sort();

    let mut from: Option<&str> = None;
    for (start, end) in &spans {
        if unknown_between(reading, from, Some(start)) {
            return true;
        }
        from = Some(match from {
            Some(held) if held > end.as_str() => held,
            _ => end,
        });
    }
    unknown_between(reading, from, None)
}

fn unknown_between(reading: &Snapshot, from: Option<&str>, to: Option<&str>) -> bool {
    if let (Some(from), Some(to)) = (from, to)
        && from >= to
    {
        return false;
    }
    let lower = from.map_or(Bound::Unbounded, Bound::Included);
    let upper = to.map_or(Bound::Unbounded, Bound::Excluded);
    reading
        .items
        .range::<str, _>((lower, upper))
        .any(|(key, _)| Kind::of(key) == Kind::Unknown)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const KNOWN: &[&str] = &[
        "unit",
        "unit|",
        "unit|nginx.service",
        "timer|certbot.timer",
        "cron|/etc/crontab|root|x",
        "module",
        "module|overlay",
        "modules",
        "modules|unreadable",
        "script|/etc/profile",
        "preload|/etc/ld.so.preload",
    ];

    const UNKNOWN: &[&str] = &[
        "",
        "|x",
        "a",
        "unit\u{0}",
        "unit\u{0}x",
        "unit!",
        "unita",
        "units|x",
        "unit{",
        "unit}",
        "unit}x",
        "modulea",
        "modules}",
        "modulet|x",
        "crons",
        "script}",
        "timer\u{0}|x",
        "initramfs|/boot/initrd",
        "zzz",
        "~",
        "\u{10FFFF}",
    ];

    fn reading(keys: &[&str]) -> Snapshot {
        let mut reading = Snapshot::new("persistence", "2026-09-14T12:00:00.000Z".to_string());
        for key in keys {
            reading.items.insert(key.to_string(), json!({}));
        }
        reading
    }

    fn walked(reading: &Snapshot) -> bool {
        reading
            .items
            .keys()
            .any(|key| Kind::of(key) == Kind::Unknown)
    }

    #[test]
    fn the_gaps_between_known_kinds_answer_whether_something_is_unknown_exactly_as_a_walk_does() {
        let mut readings = vec![reading(&[]), reading(KNOWN)];
        for probe in KNOWN.iter().chain(UNKNOWN) {
            readings.push(reading(&[probe]));
            let mut around = KNOWN.to_vec();
            around.push(probe);
            readings.push(reading(&around));
        }
        readings.push(reading(UNKNOWN));

        for reading in &readings {
            assert_eq!(
                holds_something_unknown(reading),
                walked(reading),
                "{:?}: the list of other objects must appear exactly when a walk would show it, \
                 or a newer agent's object is hidden or an empty list is offered",
                reading.items.keys().collect::<Vec<_>>()
            );
        }
    }
}
