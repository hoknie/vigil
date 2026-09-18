use vigil_model::Finding;

use crate::ui::{Screen, modules};

const SPOOL: &str = "agent.buffer";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub screen: Screen,
    pub key: String,
}

impl Anchor {
    pub fn of(finding: &Finding) -> Option<Anchor> {
        if let Some(anchor) = declared(&finding.finding_key) {
            return Some(anchor);
        }

        let (family, rest) = finding.finding_key.split_once('|')?;
        match family == SPOOL {
            true => Some(Anchor {
                screen: Screen::SUMMARY,
                key: rest.to_string(),
            }),
            false => None,
        }
    }
}

fn declared(finding_key: &str) -> Option<Anchor> {
    modules().into_iter().find_map(|module| {
        let key = module.row_of(finding_key)?;
        let screen = Screen::parse(module.section()?.name())?;

        Some(Anchor { screen, key })
    })
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

    fn onto(key: &str) -> Anchor {
        Anchor::of(&keyed(key)).unwrap_or_else(|| panic!("{key} is a finding nobody can walk to"))
    }

    fn screen(name: &str) -> Screen {
        Screen::parse(name).unwrap_or_else(|| panic!("{name} is not a section of this console"))
    }

    #[test]
    fn a_finding_about_a_socket_points_at_that_socket_on_the_ports_screen() {
        let anchor = onto("port.listen|tcp|0.0.0.0:4444");

        assert_eq!(anchor.screen, screen("network"));
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
            let anchor = onto(key);
            assert_eq!(anchor.screen, screen("accounts"));
            assert_eq!(anchor.key, expected);
        }
    }

    #[test]
    fn every_family_of_rules_has_a_section_that_holds_its_object() {
        for (key, named) in [
            ("port.listen|tcp|0.0.0.0:443", "ports"),
            ("user|account|deploy", "accounts"),
            ("process|exec|/bin/sh|www-data", "programs"),
            ("persistence|cron|/etc/crontab|root|/x", "startup"),
            ("run|alice|/usr/bin/nc", "programs"),
            ("agent.buffer|launches", "programs"),
            ("firewall|summary|nftables", "firewall"),
            ("resource|disk|/var", "system"),
            ("file|/etc/hosts", "system"),
            ("directory|/usr/bin", "system"),
            ("container|privileged|3ab1", "containers"),
        ] {
            assert_eq!(onto(key).screen, screen(named), "{key}");
        }
    }

    #[test]
    fn every_family_any_module_of_this_build_raises_can_be_walked_to() {
        for module in modules() {
            for family in module.families() {
                let key = format!("{family}|something|else");

                assert!(
                    Anchor::of(&keyed(&key)).is_some(),
                    "{} raises {family} and a reader pressing o on it is told there is \
                     nowhere to go",
                    module.name()
                );
            }
        }
    }

    #[test]
    fn a_finding_about_a_launch_points_at_the_whole_key_because_that_family_adds_no_prefix() {
        assert_eq!(
            onto("run|alice|/usr/bin/nc").key,
            "run|alice|/usr/bin/nc",
            "the launches collector keys its own rows this way; cutting the first segment \
             would look for a row that was never written"
        );
    }

    #[test]
    fn the_finding_about_a_dropping_spool_points_at_the_row_that_says_so() {
        let anchor = onto("agent.buffer|launches");

        assert_eq!(anchor.screen, screen("programs"));
        assert_eq!(
            anchor.key, "launches|dropping",
            "the finding is keyed by the buffer and its object is the row about the loss"
        );
    }

    #[test]
    fn a_buffer_no_module_owns_is_a_receiver_and_its_row_is_on_the_summary() {
        let anchor = onto("agent.buffer|ndjson");

        assert_eq!(anchor.screen, Screen::SUMMARY);
        assert_eq!(anchor.key, "ndjson");
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
            let anchor = onto(finding_key);

            assert_eq!(anchor.screen, screen("firewall"));
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
