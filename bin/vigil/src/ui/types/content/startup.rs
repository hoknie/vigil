use crate::ui::helpers::motion::step_along::step_along;
use crate::ui::screens::startup::Kind;
use crate::ui::{Choice, Reading, View};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Startup {
    #[default]
    Units,
    Timers,
    Cron,
    Modules,
    Files,
    Other,
}

impl Startup {
    pub const ALL: &'static [Startup] = &[
        Startup::Units,
        Startup::Timers,
        Startup::Cron,
        Startup::Modules,
        Startup::Files,
        Startup::Other,
    ];

    pub fn on(view: &View) -> Vec<Startup> {
        Startup::ALL
            .iter()
            .copied()
            .filter(|list| *list != Startup::Other || holds_something_unknown(view))
            .collect()
    }

    pub fn name(self) -> &'static str {
        match self {
            Startup::Units => "units",
            Startup::Timers => "timers",
            Startup::Cron => "cron",
            Startup::Modules => "modules",
            Startup::Files => "files",
            Startup::Other => "other",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Startup::Units => "UNITS",
            Startup::Timers => "TIMERS",
            Startup::Cron => "CRON",
            Startup::Modules => "MODULES",
            Startup::Files => "FILES",
            Startup::Other => "OTHER",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Startup::Units => "THE SELECTED UNIT",
            Startup::Timers => "THE SELECTED TIMER",
            Startup::Cron => "THE SELECTED CRON JOB",
            Startup::Modules => "THE SELECTED MODULE",
            Startup::Files => "THE SELECTED FILE",
            Startup::Other => "THE SELECTED OBJECT",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            Startup::Units => "every systemd unit found on disk, and the command it runs",
            Startup::Timers => "every systemd timer, when it fires and what it starts",
            Startup::Cron => "every cron job, and the whole command it runs",
            Startup::Modules => "the kernel modules loaded right now",
            Startup::Files => {
                "files whose contents other processes execute: shell profiles, rc.local and \
                 /etc/ld.so.preload"
            }
            Startup::Other => "objects of a kind this console does not know, from a newer agent",
        }
    }

    pub fn thing(self) -> &'static str {
        match self {
            Startup::Units => "unit",
            Startup::Timers => "timer",
            Startup::Cron => "cron job",
            Startup::Modules => "kernel module",
            Startup::Files => "file",
            Startup::Other => "object of an unknown kind",
        }
    }

    pub fn things(self, how_many: usize) -> String {
        if how_many == 1 {
            return self.thing().to_string();
        }
        match self {
            Startup::Units => "units".to_string(),
            Startup::Timers => "timers".to_string(),
            Startup::Cron => "cron job(s)".to_string(),
            Startup::Modules => "kernel modules".to_string(),
            Startup::Files => "files".to_string(),
            Startup::Other => "objects of a kind this console does not know".to_string(),
        }
    }

    pub fn empty(self) -> &'static str {
        match self {
            Startup::Units => "This host starts nothing from a systemd unit.",
            Startup::Timers => "No systemd timer is set on this host.",
            Startup::Cron => "No cron job was found, in any of the four places cron reads.",
            Startup::Modules => "No kernel module is loaded.",
            Startup::Files => {
                "None of the files other processes execute was found: no shell profile, no \
                 rc.local, no /etc/ld.so.preload."
            }
            Startup::Other => "Nothing of a kind this console does not know.",
        }
    }

    pub fn holding(key: &str) -> Startup {
        match Kind::of(key) {
            Kind::Unit => Startup::Units,
            Kind::Timer => Startup::Timers,
            Kind::Cron => Startup::Cron,
            Kind::Module | Kind::ModulesUnreadable => Startup::Modules,
            Kind::Script | Kind::Preload => Startup::Files,
            Kind::Unknown => Startup::Other,
        }
    }
}

impl Choice for Startup {
    const COUNT: usize = Startup::ALL.len();

    fn index(self) -> usize {
        Startup::ALL
            .iter()
            .position(|list| *list == self)
            .unwrap_or(0)
    }

    fn step(self, by: isize, shown: &[Startup]) -> Startup {
        step_along(self, by, shown)
    }
}

fn holds_something_unknown(view: &View) -> bool {
    match view.reading("persistence") {
        Reading::Taken(snapshot) => snapshot
            .items
            .keys()
            .any(|key| Kind::of(key) == Kind::Unknown),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::Snapshot;

    use super::*;
    use crate::ui::fixture;

    #[test]
    fn every_key_of_the_persistence_reading_is_a_row_of_exactly_one_list() {
        let view = fixture::view();
        let Reading::Taken(snapshot) = view.reading("persistence") else {
            panic!("the fixture holds a persistence reading");
        };

        for key in snapshot.items.keys() {
            let lists: Vec<Startup> = Startup::ALL
                .iter()
                .copied()
                .filter(|list| Startup::holding(key) == *list)
                .collect();
            assert_eq!(lists.len(), 1, "{key} belongs to {lists:?}");
        }
    }

    #[test]
    fn the_one_preload_file_is_a_row_and_not_a_list_of_its_own() {
        assert_eq!(
            Startup::holding("preload|/etc/ld.so.preload"),
            Startup::Files,
            "a list of one row teaches a reader not to press the row of lists"
        );
        assert_eq!(
            Startup::holding("script|/etc/profile"),
            Startup::Files,
            "and it lies beside the other files whose contents somebody else executes"
        );
    }

    #[test]
    fn the_five_are_always_there_and_the_sixth_only_when_it_has_something_in_it() {
        let shown = Startup::on(&fixture::view());

        assert_eq!(
            shown.iter().map(|list| list.name()).collect::<Vec<_>>(),
            vec!["units", "timers", "cron", "modules", "files"]
        );

        let mut view = fixture::view();
        view.readings.put(
            "persistence",
            Reading::Taken(
                Snapshot::new("persistence", "2026-09-09T09:00:00.000Z").with(
                    "initramfs|/boot/initrd.img",
                    json!({"from": "a newer agent"}),
                ),
            ),
        );
        assert!(Startup::on(&view).contains(&Startup::Other));
    }

    #[test]
    fn each_one_says_what_it_is_and_what_it_says_when_it_is_empty() {
        for list in Startup::ALL {
            assert!(!list.about().is_empty(), "{}", list.name());
            assert!(!list.empty().is_empty(), "{}", list.name());
            assert!(!list.detail().is_empty(), "{}", list.name());
        }
    }
}
