use vigil_model::Finding;

use crate::ui::Screen;

pub const SOCKETS: &str = "port.listen";

pub const ACCOUNTS: &str = "user";

pub const PROCESSES: &str = "process";

pub const PERSISTENCE: &str = "persistence";

pub const LAUNCHES: &str = "run";

pub const SPOOL: &str = "agent.buffer";

pub const FIREWALL: &str = "firewall";

pub const RESOURCE: &str = "resource";

pub const CONTAINER: &str = "container";

pub const FILE: &str = "file";

pub const DIRECTORY: &str = "directory";

const DROPPING: &str = "launches|dropping";

const AUDIT_SPOOL: &str = "launches";

enum Reach {
    Rest,
    Whole,
    Prefixed(&'static str),
    OfTheHost,
    Named,
}

const TABLE: &[(&str, Screen, Reach)] = &[
    (SOCKETS, Screen::Ports, Reach::Rest),
    (ACCOUNTS, Screen::Accounts, Reach::Rest),
    (PROCESSES, Screen::Programs, Reach::Rest),
    (PERSISTENCE, Screen::Startup, Reach::Rest),
    (LAUNCHES, Screen::Programs, Reach::Whole),
    (FIREWALL, Screen::Firewall, Reach::Prefixed(FIREWALL_ROW)),
    (RESOURCE, Screen::System, Reach::OfTheHost),
    (CONTAINER, Screen::Containers, Reach::Named),
    (FILE, Screen::System, Reach::Whole),
    (DIRECTORY, Screen::System, Reach::Whole),
];

const BOOT_ROW: &str = "boot|current";

const FILESYSTEM_ROW: &str = "fs";

const FIREWALL_ROW: &str = "fw-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub screen: Screen,
    pub key: String,
}

impl Anchor {
    pub fn of(finding: &Finding) -> Option<Anchor> {
        let (family, rest) = finding.finding_key.split_once('|')?;
        if family == SPOOL {
            return Some(match rest {
                AUDIT_SPOOL => Anchor {
                    screen: Screen::Programs,
                    key: DROPPING.to_string(),
                },
                receiver => Anchor {
                    screen: Screen::Summary,
                    key: receiver.to_string(),
                },
            });
        }
        let (_, screen, reach) = TABLE.iter().find(|(named, _, _)| *named == family)?;

        Some(Anchor {
            screen: *screen,
            key: match reach {
                Reach::Rest => rest.to_string(),
                Reach::Whole => finding.finding_key.clone(),
                Reach::Prefixed(prefix) => format!("{prefix}{rest}"),
                Reach::OfTheHost => match rest.split_once('|') {
                    Some((_, mount)) => format!("{FILESYSTEM_ROW}|{mount}"),
                    None => BOOT_ROW.to_string(),
                },
                Reach::Named => rest
                    .split_once('|')
                    .map(|(_, named)| named.to_string())
                    .unwrap_or_else(|| rest.to_string()),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    fn keyed(key: &str) -> Finding {
        let mut finding = fixture::finding("something happened", Severity::Low);
        finding.finding_key = key.to_string();
        finding
    }

    #[test]
    fn a_finding_about_a_socket_points_at_that_socket_on_the_ports_screen() {
        let anchor = Anchor::of(&keyed("port.listen|tcp|0.0.0.0:4444")).expect("a socket");

        assert_eq!(anchor.screen, Screen::Ports);
        assert_eq!(
            anchor.key, "tcp|0.0.0.0:4444",
            "the key is the collector's, so the row on the far screen is found by it"
        );
    }

    #[test]
    fn a_finding_about_an_account_a_group_or_a_key_points_at_the_accounts_screen() {
        for (key, expected) in [
            ("user|account|backdoor", "account|backdoor"),
            ("user|group|docker", "group|docker"),
            ("user|sshkey|deploy|SHA256:abc", "sshkey|deploy|SHA256:abc"),
        ] {
            let anchor = Anchor::of(&keyed(key)).expect(key);
            assert_eq!(anchor.screen, Screen::Accounts);
            assert_eq!(anchor.key, expected);
        }
    }

    #[test]
    fn every_family_of_rules_has_a_section_that_holds_its_object() {
        for (key, screen) in [
            ("port.listen|tcp|0.0.0.0:443", Screen::Ports),
            ("user|account|deploy", Screen::Accounts),
            ("process|exec|/bin/sh|www-data", Screen::Programs),
            ("persistence|cron|/etc/crontab|root|/x", Screen::Startup),
            ("run|alice|/usr/bin/nc", Screen::Programs),
            ("agent.buffer|launches", Screen::Programs),
        ] {
            let anchor = Anchor::of(&keyed(key))
                .unwrap_or_else(|| panic!("{key} is a finding a reader cannot walk to"));
            assert_eq!(anchor.screen, screen, "{key}");
        }
    }

    #[test]
    fn a_finding_about_a_launch_points_at_the_whole_key_because_that_family_adds_no_prefix() {
        let anchor = Anchor::of(&keyed("run|alice|/usr/bin/nc")).expect("a launch");

        assert_eq!(
            anchor.key, "run|alice|/usr/bin/nc",
            "the launches collector keys its own rows this way; cutting the first segment \
             would look for a row that was never written"
        );
    }

    #[test]
    fn the_finding_about_a_dropping_spool_points_at_the_row_that_says_so() {
        let anchor = Anchor::of(&keyed("agent.buffer|launches")).expect("a spool");

        assert_eq!(anchor.screen, Screen::Programs);
        assert_eq!(
            anchor.key, "launches|dropping",
            "the finding is keyed by the buffer and its object is the row about the loss"
        );
    }

    #[test]
    fn every_row_of_the_firewall_reading_is_reached_from_its_finding_by_one_rule_and_not_four() {
        for (finding_key, row) in [
            ("firewall|summary|nftables", "fw-summary|nftables"),
            ("firewall|table|inet filter", "fw-table|inet filter"),
            (
                "firewall|chain|inet filter|input",
                "fw-chain|inet filter|input",
            ),
            ("firewall|backend|legacy", "fw-backend|legacy"),
        ] {
            let anchor = Anchor::of(&keyed(finding_key))
                .unwrap_or_else(|| panic!("{finding_key} is a finding a reader cannot walk to"));

            assert_eq!(anchor.screen, Screen::Firewall);
            assert_eq!(
                anchor.key, row,
                "the rule reads the whole family: the finding names it with the word the \
                 contract uses and the reading with the short one, and one substitution \
                 turns either into the other. Four of them would be four places to forget"
            );
        }
    }

    #[test]
    fn a_finding_about_the_agent_itself_has_no_object_on_this_host_to_walk_to() {
        assert_eq!(Anchor::of(&keyed("agent.collector|ports")), None);
        assert_eq!(Anchor::of(&keyed("agent.budget|cpu")), None);
        assert_eq!(Anchor::of(&keyed("agent.store|findings")), None);
        assert_eq!(Anchor::of(&keyed("nothing-shaped-like-a-key")), None);
    }
}
