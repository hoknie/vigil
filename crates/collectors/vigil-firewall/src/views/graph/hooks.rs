use serde_json::Value;
use vigil_model::Snapshot;

use crate::types::Kind;
use crate::views::fields;

pub(super) const WALKED: &[&str] = &[
    "ingress",
    "prerouting",
    "input",
    "forward",
    "output",
    "postrouting",
];

const NO_CHAIN: &str = "no chain here";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OnTheHook {
    pub hook: &'static str,
    pub chain: String,
    pub policy: String,
    pub rules: u64,
}

impl OnTheHook {
    pub(super) fn walked(reading: &Snapshot) -> Vec<OnTheHook> {
        WALKED
            .iter()
            .map(|hook| OnTheHook::of(reading, hook))
            .collect()
    }

    pub(super) fn of(reading: &Snapshot, hook: &'static str) -> OnTheHook {
        let found: Vec<(&String, &Value)> = reading
            .items
            .iter()
            .filter(|(key, item)| {
                Kind::of(key) == Some(Kind::Chain) && item["hook"].as_str() == Some(hook)
            })
            .collect();

        match found.first() {
            None => OnTheHook {
                hook,
                chain: NO_CHAIN.to_string(),
                policy: String::new(),
                rules: 0,
            },
            Some((_, item)) => OnTheHook {
                hook,
                chain: named(item, found.len()),
                policy: item["policy"].as_str().unwrap_or_default().to_string(),
                rules: found.iter().map(|(_, item)| fields::all_rules(item)).sum(),
            },
        }
    }

    pub(super) fn holds_a_chain(&self) -> bool {
        self.chain != NO_CHAIN
    }
}

fn named(item: &Value, found: usize) -> String {
    let one = format!(
        "{} {} · {}",
        item["family"].as_str().unwrap_or("?"),
        item["table"].as_str().unwrap_or("?"),
        item["name"].as_str().unwrap_or("?")
    );

    match found {
        1 => one,
        many => format!("{one} and {} more", many - 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::firewall;

    #[test]
    fn every_hook_a_packet_passes_is_walked_in_the_order_the_kernel_walks_them() {
        assert_eq!(
            WALKED,
            &[
                "ingress",
                "prerouting",
                "input",
                "forward",
                "output",
                "postrouting"
            ],
            "the graph is read top to bottom, and a hook out of order draws a path no packet \
             takes"
        );
    }

    #[test]
    fn a_hook_with_a_chain_on_it_is_shown_with_the_policy_that_chain_holds() {
        let walked = OnTheHook::walked(&firewall());
        let input = walked
            .iter()
            .find(|hook| hook.hook == "input")
            .expect("input is walked");

        assert!(input.holds_a_chain());
        assert_eq!(input.chain, "inet filter · input");
        assert_eq!(input.policy, "drop");
        assert_eq!(input.rules, 3);
    }

    #[test]
    fn a_hook_no_chain_of_this_host_sits_on_says_so_rather_than_being_left_off_the_path() {
        let walked = OnTheHook::walked(&firewall());
        let ingress = walked
            .iter()
            .find(|hook| hook.hook == "ingress")
            .expect("ingress is walked");

        assert!(
            !ingress.holds_a_chain(),
            "a hook missing from the drawing reads as a hook a packet does not pass, and \
             every packet passes all of them: what is missing is the chain, not the hook"
        );
        assert_eq!(ingress.rules, 0);
    }

    #[test]
    fn two_chains_on_one_hook_are_both_counted_and_the_drawing_says_there_is_more_than_one() {
        let mut crowded = firewall();
        crowded.items.insert(
            "fw-chain|inet firewalld|filter_INPUT".to_string(),
            serde_json::json!({
                "family": "inet", "table": "firewalld", "name": "filter_INPUT",
                "type": "filter", "hook": "input", "priority": 10, "policy": "accept",
                "rules": 4, "rules_kept": []
            }),
        );

        let input = OnTheHook::of(&crowded, "input");

        assert!(input.chain.contains("and 1 more"), "{}", input.chain);
        assert_eq!(input.rules, 7);
    }
}
