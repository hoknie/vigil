use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::application_firewall::ApplicationFirewall;
use super::nft_json::RULES_KEPT_PER_CHAIN;
use super::pf::{Direction, MAIN, PfRule, PfRuleset};
use super::reading::{SOURCE, add_interfaces};
use crate::types::Interface;

pub const PF_SUMMARY: &str = "fw-summary|pf";

pub const APPLICATION_FIREWALL_ROW: &str = "fw-application|socketfilterfw";

const TABLE: &str = "fw-table";

const CHAIN: &str = "fw-chain";

const FAMILY: &str = "pf";

const INTERFACE: &str = "fw-interface|";

const NOT_COUNTED_ON_A_MAC: &str = "dropped_out";

pub struct Pf<'a> {
    pub enabled: bool,
    pub main: &'a PfRuleset,
    pub anchors: &'a [PfRuleset],
}

pub struct MacosFirewallReading<'a> {
    pub pf: Option<Pf<'a>>,
    pub application_firewall: Option<&'a ApplicationFirewall>,
    pub interfaces: &'a [Interface],
    pub counting: bool,
}

struct Chain<'a> {
    name: &'static str,
    kind: &'static str,
    hook: &'static str,
    policy: &'static str,
    rules: Vec<&'a PfRule>,
}

pub fn macos_firewall_snapshot(taken_at: &str, reading: &MacosFirewallReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    if let Some(pf) = &reading.pf {
        add_pf(&mut snapshot, pf);
    }
    if let Some(application_firewall) = reading.application_firewall {
        add_application_firewall(&mut snapshot, application_firewall);
    }
    add_interfaces(&mut snapshot, reading.interfaces, reading.counting);
    for (_, row) in snapshot
        .items
        .iter_mut()
        .filter(|(key, _)| key.starts_with(INTERFACE))
    {
        if let Some(fields) = row.as_object_mut() {
            fields.remove(NOT_COUNTED_ON_A_MAC);
        }
    }

    snapshot
}

fn add_pf(snapshot: &mut Snapshot, pf: &Pf<'_>) {
    let chains = chains_of(pf.main);
    let hooked_on_input = chains
        .iter()
        .filter(|chain| chain.hook == "input" && !chain.rules.is_empty())
        .count();
    let rules = pf.main.rules() + pf.anchors.iter().map(PfRuleset::rules).sum::<usize>();

    snapshot.items.insert(
        PF_SUMMARY.to_string(),
        json!({
            "enabled": pf.enabled,
            "families": [FAMILY],
            "tables": 1 + pf.anchors.len(),
            "chains": chains.len() + pf.anchors.len(),
            "base_chains": chains.len(),
            "anchors": pf.anchors.len(),
            "rules": rules,
            "hooked_on_input": match pf.enabled {
                true => hooked_on_input,
                false => 0,
            },
            "legacy_backend": false,
        }),
    );

    snapshot.items.insert(
        format!("{TABLE}|{FAMILY} {MAIN}"),
        json!({
            "family": FAMILY,
            "name": MAIN,
            "chains": chains.len(),
            "rules": pf.main.rules(),
            "anchor": false,
        }),
    );
    for anchor in pf.anchors {
        snapshot.items.insert(
            format!("{TABLE}|{FAMILY} {}", anchor.name),
            json!({
                "family": FAMILY,
                "name": anchor.name,
                "chains": 1,
                "rules": anchor.rules(),
                "anchor": true,
            }),
        );
    }

    for chain in chains {
        snapshot.items.insert(
            format!("{CHAIN}|{FAMILY} {MAIN}|{}", chain.name),
            json!({
                "family": FAMILY,
                "table": MAIN,
                "name": chain.name,
                "type": chain.kind,
                "hook": chain.hook,
                "priority": 0,
                "policy": chain.policy,
                "rules": chain.rules.len(),
                "rules_kept": chain
                    .rules
                    .iter()
                    .take(RULES_KEPT_PER_CHAIN)
                    .map(|rule| json!({
                        "handle": rule.number,
                        "matches": rule.matches,
                        "does": rule.does(),
                    }))
                    .collect::<Vec<Value>>(),
            }),
        );
    }
}

fn chains_of(main: &PfRuleset) -> Vec<Chain<'_>> {
    let mut chains = vec![
        Chain {
            name: "in",
            kind: "filter",
            hook: "input",
            policy: main.policy(Direction::In),
            rules: main.filtering(Direction::In),
        },
        Chain {
            name: "out",
            kind: "filter",
            hook: "output",
            policy: main.policy(Direction::Out),
            rules: main.filtering(Direction::Out),
        },
    ];

    let redirecting = main.redirecting();
    if !redirecting.is_empty() {
        chains.push(Chain {
            name: "rdr",
            kind: "nat",
            hook: "prerouting",
            policy: "accept",
            rules: redirecting,
        });
    }
    let translating = main.translating_out();
    if !translating.is_empty() {
        chains.push(Chain {
            name: "nat",
            kind: "nat",
            hook: "postrouting",
            policy: "accept",
            rules: translating,
        });
    }

    chains
}

fn add_application_firewall(snapshot: &mut Snapshot, read: &ApplicationFirewall) {
    let allowed = read
        .applications
        .iter()
        .filter(|(_, allowed)| *allowed)
        .count();

    snapshot.items.insert(
        APPLICATION_FIREWALL_ROW.to_string(),
        json!({
            "enabled": read.enabled().unwrap_or(false),
            "state": read.state,
            "blocks_all": read.blocks_all.unwrap_or(false),
            "stealth": read.stealth.unwrap_or(false),
            "allows_signed": read.allows_signed.unwrap_or(false),
            "allows_downloaded_signed": read.allows_downloaded_signed.unwrap_or(false),
            "applications_allowed": allowed,
            "applications_blocked": read.applications.len() - allowed,
            "applications": read
                .applications
                .iter()
                .map(|(path, allowed)| json!({"path": path, "allowed": allowed}))
                .collect::<Vec<Value>>(),
        }),
    );
}
