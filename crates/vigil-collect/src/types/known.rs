#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownCollector {
    pub name: &'static str,
    pub subject: &'static str,
    pub every_seconds: u32,
}

pub const COLLECTORS: &[KnownCollector] = &[
    KnownCollector {
        name: "ports",
        subject: "the sockets this host listens on, and the process holding each one",
        every_seconds: 30,
    },
    KnownCollector {
        name: "users",
        subject: "who may log in to this host, as whom, and with what",
        every_seconds: 300,
    },
    KnownCollector {
        name: "persistence",
        subject: "what the host starts by itself: units, timers, cron, shell profiles",
        every_seconds: 300,
    },
    KnownCollector {
        name: "processes",
        subject: "the programs running on this host",
        every_seconds: 30,
    },
    KnownCollector {
        name: "launches",
        subject: "what people run, from the kernel's audit records",
        every_seconds: 15,
    },
];

pub fn is_known(name: &str) -> bool {
    COLLECTORS.iter().any(|collector| collector.name == name)
}

pub fn subject_of(name: &str) -> Option<&'static str> {
    COLLECTORS
        .iter()
        .find(|collector| collector.name == name)
        .map(|collector| collector.subject)
}

pub fn every_seconds_of(name: &str) -> Option<u32> {
    COLLECTORS
        .iter()
        .find(|collector| collector.name == name)
        .map(|collector| collector.every_seconds)
}

pub fn names() -> Vec<&'static str> {
    COLLECTORS.iter().map(|collector| collector.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use vigil_model::{Golden, Settled};

    fn declared() -> Settled {
        let mut pinned = Settled::new("collectors");

        for collector in COLLECTORS {
            pinned = pinned.pinning(
                collector.name,
                "every_seconds",
                collector.every_seconds,
                format!(
                    "the period {} is declared with among the collectors this build ships",
                    collector.name
                ),
            );
        }

        pinned
    }

    #[test]
    fn the_period_a_collector_declares_is_published_for_the_screens_that_show_it() {
        if let Err(complaint) = Golden::settled("collectors").write_or_check(&declared().written())
        {
            panic!("{complaint}");
        }
    }

    #[test]
    fn a_name_is_in_the_list_once_and_says_what_it_watches() {
        let mut seen = names();
        let total = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), total, "two collectors share a name");

        for collector in COLLECTORS {
            assert!(!collector.name.is_empty());
            assert!(
                collector.subject.len() > 10,
                "{} says nothing about what it watches",
                collector.name
            );
        }
    }

    #[test]
    fn a_name_nobody_has_is_not_known() {
        assert!(is_known("ports"));
        assert!(!is_known("proccesses"));
        assert_eq!(subject_of("proccesses"), None);
        assert_eq!(every_seconds_of("proccesses"), None);
    }

    #[test]
    fn every_collector_declares_how_often_its_subject_is_worth_reading() {
        for collector in COLLECTORS {
            assert!(
                collector.every_seconds > 0,
                "{} would be read without a pause",
                collector.name
            );
            assert!(
                collector.every_seconds <= 3_600,
                "{} would be read less than once an hour",
                collector.name
            );
        }

        assert_eq!(every_seconds_of("launches"), Some(15));
        assert_eq!(every_seconds_of("ports"), Some(30));
        assert_eq!(every_seconds_of("processes"), Some(30));
        assert_eq!(every_seconds_of("users"), Some(300));
        assert_eq!(every_seconds_of("persistence"), Some(300));
    }

    #[test]
    fn a_subject_that_outlives_a_reboot_is_read_less_often_than_one_that_does_not() {
        let transient = every_seconds_of("ports").expect("ports");
        let lasting = every_seconds_of("persistence").expect("persistence");

        assert!(
            transient < lasting,
            "a socket exists between two readings or it never existed; a unit waits"
        );
    }
}
