use serde_json::Value;
use vigil_model::Snapshot;

use super::kind::Kind;

const ENTERED_BY: &[&str] = &[
    "filter_IN_",
    "filter_OUT_",
    "filter_FWDI_",
    "filter_FWDO_",
    "nat_PRE_",
    "nat_POST_",
    "mangle_PRE_",
];

const A_PART_OF_ONE: &[&str] = &["_pre", "_post", "_log", "_deny", "_allow"];

const ON_AN_INTERFACE: &str = "meta iifname ";

const OR_LEAVING_ON_ONE: &str = "meta oifname ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone {
    pub name: String,
    pub on: Vec<String>,
    pub through: Vec<String>,
}

impl Zone {
    pub fn of(reading: &Snapshot) -> Vec<Zone> {
        let mut zones: Vec<Zone> = Vec::new();
        let nothing: Vec<Value> = Vec::new();

        for (_, item) in reading
            .items
            .iter()
            .filter(|(key, _)| Kind::of(key) == Some(Kind::Chain))
        {
            for rule in item["rules_kept"].as_array().unwrap_or(&nothing) {
                let Some((name, chain)) = entered(rule) else {
                    continue;
                };
                let at = match zones.iter().position(|zone| zone.name == name) {
                    Some(at) => at,
                    None => {
                        zones.push(Zone {
                            name,
                            on: Vec::new(),
                            through: Vec::new(),
                        });
                        zones.len() - 1
                    }
                };
                note(&mut zones[at].through, chain);
                if let Some(interface) = interface_of(rule) {
                    note(&mut zones[at].on, interface);
                }
            }
        }

        zones.sort_by(|left, right| left.name.cmp(&right.name));
        zones
    }

    pub fn shown_on(&self) -> String {
        match self.on.is_empty() {
            true => "any interface".to_string(),
            false => self.on.join(", "),
        }
    }
}

fn entered(rule: &Value) -> Option<(String, &str)> {
    let does = rule["does"].as_str()?;
    let chain = does
        .strip_prefix("jump ")
        .or_else(|| does.strip_prefix("goto "))?
        .split(' ')
        .next()?;

    let named = ENTERED_BY
        .iter()
        .find_map(|start| chain.strip_prefix(start))?;
    let named = A_PART_OF_ONE
        .iter()
        .find_map(|tail| named.strip_suffix(tail))
        .unwrap_or(named);

    match named.is_empty() {
        true => None,
        false => Some((named.to_string(), chain)),
    }
}

fn interface_of(rule: &Value) -> Option<&str> {
    let matches = rule["matches"].as_str()?;
    let at = matches
        .find(ON_AN_INTERFACE)
        .map(|at| at + ON_AN_INTERFACE.len())
        .or_else(|| {
            matches
                .find(OR_LEAVING_ON_ONE)
                .map(|at| at + OR_LEAVING_ON_ONE.len())
        })?;

    matches[at..]
        .split(' ')
        .next()
        .filter(|name| !name.is_empty())
}

fn note(known: &mut Vec<String>, said: &str) {
    if !known.iter().any(|held| held == said) {
        known.push(said.to_string());
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn a_host_behind_firewalld() -> Snapshot {
        Snapshot::new("firewall", "2026-09-16T12:00:00.000Z").with(
            "fw-chain|inet firewalld|filter_INPUT",
            json!({
                "family": "inet", "table": "firewalld", "name": "filter_INPUT",
                "type": "filter", "hook": "input", "priority": 10, "policy": "accept",
                "rules": 4,
                "rules_kept": [
                    {"handle": 1, "matches": "ct state { established, related }", "does": "accept"},
                    {"handle": 2, "matches": "meta iifname eth0", "does": "jump filter_IN_public"},
                    {"handle": 3, "matches": "meta iifname eth1", "does": "jump filter_IN_public"},
                    {"handle": 4, "matches": "meta iifname docker0", "does": "jump filter_IN_trusted_allow"}
                ]
            }),
        )
    }

    #[test]
    fn a_zone_is_the_chain_a_packet_is_sent_into_because_of_the_interface_it_arrived_on() {
        let zones = Zone::of(&a_host_behind_firewalld());

        assert_eq!(zones.len(), 2);
        assert_eq!(zones[0].name, "public");
        assert_eq!(zones[0].on, vec!["eth0", "eth1"]);
        assert_eq!(zones[0].through, vec!["filter_IN_public"]);
        assert_eq!(zones[1].name, "trusted");
    }

    #[test]
    fn a_host_whose_backend_has_no_zones_at_all_is_shown_as_having_none() {
        let plain = Snapshot::new("firewall", "2026-09-16T12:00:00.000Z").with(
            "fw-chain|inet filter|input",
            json!({
                "name": "input", "rules": 2,
                "rules_kept": [
                    {"handle": 1, "matches": "tcp dport 22", "does": "jump allowed"},
                    {"handle": 2, "matches": "anything", "does": "accept"}
                ]
            }),
        );

        assert!(
            Zone::of(&plain).is_empty(),
            "nftables has no zones of its own: one appears here only where firewalld built it \
             out of chains, and inventing one for a host without firewalld would name a thing \
             an operator cannot go and look at"
        );
    }

    #[test]
    fn a_zone_reached_without_naming_an_interface_says_it_is_reached_from_any_of_them() {
        let anywhere = Snapshot::new("firewall", "2026-09-16T12:00:00.000Z").with(
            "fw-chain|inet firewalld|filter_INPUT",
            json!({
                "name": "filter_INPUT", "rules": 1,
                "rules_kept": [{"handle": 1, "matches": "anything", "does": "goto filter_IN_home"}]
            }),
        );

        let zones = Zone::of(&anywhere);

        assert_eq!(zones[0].name, "home");
        assert_eq!(zones[0].shown_on(), "any interface");
    }

    #[test]
    fn the_chains_firewalld_builds_a_zone_out_of_are_one_zone_and_not_five() {
        let parts = Snapshot::new("firewall", "2026-09-16T12:00:00.000Z").with(
            "fw-chain|inet firewalld|filter_INPUT",
            json!({
                "name": "filter_INPUT", "rules": 3,
                "rules_kept": [
                    {"handle": 1, "matches": "meta iifname eth0", "does": "jump filter_IN_public_pre"},
                    {"handle": 2, "matches": "meta iifname eth0", "does": "jump filter_IN_public_log"},
                    {"handle": 3, "matches": "meta iifname eth0", "does": "jump filter_IN_public_deny"}
                ]
            }),
        );

        let zones = Zone::of(&parts);

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].through.len(), 3);
        assert_eq!(
            zones[0].on,
            vec!["eth0"],
            "one interface named on three of a zone's chains is one interface in that zone"
        );
    }
}
