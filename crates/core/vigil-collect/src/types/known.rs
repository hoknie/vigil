#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownCollector {
    pub name: &'static str,
    pub subject: &'static str,
    pub every_seconds: u32,
    pub unit: Option<&'static str>,
}

pub const COLLECTORS: &[KnownCollector] = &[
    KnownCollector {
        name: "processes",
        subject: "the programs running on this host",
        every_seconds: 30,
        unit: None,
    },
    KnownCollector {
        name: "resources",
        subject: "the boot this host is running, the moment it started and the size of it",
        every_seconds: 60,
        unit: None,
    },
    KnownCollector {
        name: "files",
        subject: "the files this host is configured by, and whether any of them changed",
        every_seconds: 300,
        unit: None,
    },
    KnownCollector {
        name: "launches",
        subject: "what people run, from the kernel's audit records",
        every_seconds: 15,
        unit: None,
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

pub fn unit_of(name: &str) -> Option<&'static str> {
    COLLECTORS
        .iter()
        .find(|collector| collector.name == name)
        .and_then(|collector| collector.unit)
}

pub fn names() -> Vec<&'static str> {
    COLLECTORS.iter().map(|collector| collector.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_collector_that_needs_something_running_on_the_host_names_it_rather_than_a_command_guessing()
     {
        assert_eq!(unit_of("nothing-of-ours"), None);
        assert_eq!(
            unit_of("processes"),
            None,
            "a collector that reads /proc needs nothing started for it, and a command that \
             asked a list of its own would have to be edited every time one of these appears"
        );
        assert_eq!(unit_of("nothing-of-ours"), None);

        for collector in COLLECTORS {
            if let Some(unit) = collector.unit {
                assert!(
                    unit.ends_with(".timer") || unit.ends_with(".service"),
                    "{} names {unit}, which systemd would not know",
                    collector.name
                );
            }
        }
    }

    #[test]
    fn a_name_nobody_has_is_not_known() {
        assert!(is_known("processes"));
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
        assert_eq!(every_seconds_of("processes"), Some(30));
        assert_eq!(every_seconds_of("resources"), Some(60));
        assert_eq!(every_seconds_of("files"), Some(300));
    }
}
