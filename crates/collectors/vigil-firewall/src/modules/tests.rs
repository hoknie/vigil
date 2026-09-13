use vigil_module::{Module, Settings};

use super::Firewall;

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_the_ruleset_walks_to_the_row_of_the_reading_it_is_about() {
    for (key, row) in [
        ("firewall|summary|nftables", "fw-summary|nftables"),
        ("firewall|table|inet filter", "fw-table|inet filter"),
        (
            "firewall|chain|inet filter|input",
            "fw-chain|inet filter|input",
        ),
        ("firewall|backend|legacy", "fw-backend|legacy"),
    ] {
        assert_eq!(
            Firewall.row_of(key),
            Some(row.to_string()),
            "the finding names the family with the word the contract uses and the reading \
             with the short one, and one substitution turns either into the other: {key}"
        );
    }
    assert!(!Firewall.raised("port.listen|tcp|0.0.0.0:443"));
}

#[test]
fn reading_the_ruleset_needs_something_running_on_the_host_and_the_module_names_it() {
    assert_eq!(
        Firewall.unit(),
        Some("vigil-firewall.timer"),
        "nft is not run by this daemon: a timer writes the ruleset down and the collector \
         reads what it wrote"
    );
    assert_eq!(Firewall.name(), "firewall");
    assert_eq!(Firewall.every_seconds(), 60);
    assert!(!Firewall.rules(&Settings::plain(at_noon)).is_empty());
}
