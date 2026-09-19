use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Ruleset,
    Application,
    Table,
    Chain,
    Backend,
    Interface,
}

pub struct FirewallView<'a> {
    key: &'a str,
    value: &'a Value,
}

const SUMMARY: &str = "fw-summary";

const TABLE: &str = "fw-table";

const CHAIN: &str = "fw-chain";

const BACKEND: &str = "fw-backend";

const INTERFACE: &str = "fw-interface";

const APPLICATION: &str = "fw-application";

const PF: &str = "pf";

const CLOSED: &str = "drop";

const OPEN: &str = "accept";

impl<'a> FirewallView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        FirewallView { key, value }
    }

    pub fn family(&self) -> Option<Family> {
        match self.key.split('|').next()? {
            SUMMARY => Some(Family::Ruleset),
            TABLE => Some(Family::Table),
            CHAIN => Some(Family::Chain),
            BACKEND => Some(Family::Backend),
            INTERFACE => Some(Family::Interface),
            APPLICATION => Some(Family::Application),
            _ => None,
        }
    }

    pub fn is(&self, family: Family) -> bool {
        self.family() == Some(family)
    }

    pub fn tables(&self) -> u64 {
        self.number("tables")
    }

    pub fn rules(&self) -> u64 {
        self.number("rules")
    }

    pub fn chains(&self) -> u64 {
        self.number("chains")
    }

    pub fn hooked_on_input(&self) -> u64 {
        self.number("hooked_on_input")
    }

    pub fn backend(&self) -> &'a str {
        match self.family() {
            Some(Family::Ruleset) => self.key.split_once('|').map_or("?", |(_, backend)| backend),
            Some(Family::Application) => "application firewall",
            _ => "?",
        }
    }

    pub fn is_pf(&self) -> bool {
        self.backend() == PF
    }

    pub fn enabled(&self) -> bool {
        self.value["enabled"].as_bool().unwrap_or(false)
    }

    pub fn blocks_all(&self) -> bool {
        self.value["blocks_all"].as_bool().unwrap_or(false)
    }

    pub fn is_an_anchor(&self) -> bool {
        self.value["anchor"].as_bool().unwrap_or(false)
    }

    pub fn legacy_backend(&self) -> bool {
        self.value["legacy_backend"].as_bool().unwrap_or(false)
    }

    pub fn families(&self) -> Vec<&'a str> {
        self.value["families"]
            .as_array()
            .map(|values| values.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    pub fn name(&self) -> &'a str {
        self.value["name"].as_str().unwrap_or("?")
    }

    pub fn table(&self) -> &'a str {
        self.value["table"].as_str().unwrap_or("?")
    }

    pub fn network_family(&self) -> &'a str {
        self.value["family"].as_str().unwrap_or("?")
    }

    pub fn hook(&self) -> &'a str {
        self.value["hook"].as_str().unwrap_or("?")
    }

    pub fn policy(&self) -> &'a str {
        self.value["policy"].as_str().unwrap_or("?")
    }

    pub fn drops_what_no_rule_allowed(&self) -> bool {
        self.policy() == CLOSED
    }

    pub fn accepts_what_no_rule_allowed(&self) -> bool {
        self.policy() == OPEN
    }

    fn number(&self, field: &str) -> u64 {
        self.value[field].as_u64().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::fixture as neighbours;

    #[test]
    fn an_item_of_another_collector_belongs_to_no_family_here() {
        let socket = neighbours::of_another_collector("tcp|0.0.0.0:443");

        assert_eq!(FirewallView::new("tcp|0.0.0.0:443", &socket).family(), None);
        assert_eq!(
            FirewallView::new("unit|nginx.service", &socket).family(),
            None
        );
    }

    #[test]
    fn the_summary_a_table_a_chain_and_the_old_backend_are_four_different_things() {
        let summary = fixture::firewall_ruleset(2, 1, 12);
        let table = fixture::firewall_table("inet", "filter", 3, 9);
        let chain = fixture::firewall_chain("inet", "filter", "input", "drop");
        let backend = fixture::firewall_legacy_backend(&["filter", "nat"]);

        assert!(FirewallView::new("fw-summary|nftables", &summary).is(Family::Ruleset));
        assert!(FirewallView::new("fw-table|inet filter", &table).is(Family::Table));
        assert!(FirewallView::new("fw-chain|inet filter|input", &chain).is(Family::Chain));
        assert!(FirewallView::new("fw-backend|legacy", &backend).is(Family::Backend));
    }

    #[test]
    fn the_ruleset_names_the_backend_that_holds_it_and_the_application_firewall_is_its_own() {
        let pf = fixture::pf_ruleset(true, 1);
        let application = fixture::application_firewall(true, false);

        assert_eq!(
            FirewallView::new("fw-summary|nftables", &pf).backend(),
            "nftables"
        );
        assert!(FirewallView::new("fw-summary|pf", &pf).is_pf());
        assert!(
            FirewallView::new("fw-application|socketfilterfw", &application)
                .is(Family::Application)
        );
        assert!(FirewallView::new("fw-application|socketfilterfw", &application).enabled());
    }

    #[test]
    fn a_chain_that_drops_what_no_rule_allowed_is_not_one_that_accepts_it() {
        let closed = fixture::firewall_chain("inet", "filter", "input", "drop");
        let open = fixture::firewall_chain("inet", "filter", "input", "accept");

        assert!(
            FirewallView::new("fw-chain|inet filter|input", &closed).drops_what_no_rule_allowed()
        );
        assert!(
            !FirewallView::new("fw-chain|inet filter|input", &closed)
                .accepts_what_no_rule_allowed()
        );
        assert!(
            FirewallView::new("fw-chain|inet filter|input", &open).accepts_what_no_rule_allowed()
        );
    }

    #[test]
    fn a_summary_from_a_host_whose_rules_the_old_backend_holds_says_so() {
        let mut summary = fixture::firewall_ruleset(0, 0, 0);
        summary["legacy_backend"] = serde_json::json!(true);

        assert!(FirewallView::new("fw-summary|nftables", &summary).legacy_backend());
        assert!(
            !FirewallView::new("fw-summary|nftables", &fixture::firewall_ruleset(1, 1, 1))
                .legacy_backend()
        );
    }
}
