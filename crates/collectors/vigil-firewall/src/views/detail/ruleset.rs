use serde_json::Value;
use vigil_view::Piece;

use super::super::fields;

pub(super) fn ruleset(key: &str, item: &Value) -> Vec<Piece> {
    if key.ends_with("|pf") {
        return pf(key, item);
    }
    let mut said = vec![
        Piece::title(
            "NFTABLES",
            fields::version(item).unwrap_or("version unknown"),
        ),
        Piece::Blank,
    ];

    for (name, value) in [
        ("tables", fields::tables(item).to_string()),
        ("chains", fields::all_chains(item).to_string()),
        ("on a hook", fields::base_chains(item).to_string()),
        ("on input", fields::hooked_on_input(item).to_string()),
        ("rules", fields::all_rules(item).to_string()),
        (
            "families",
            match fields::families(item).is_empty() {
                true => "none".to_string(),
                false => fields::families(item).join(", "),
            },
        ),
        ("object", key.to_string()),
    ] {
        said.push(Piece::field(name, value));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(
        match (fields::legacy_backend(item), fields::hooked_on_input(item)) {
            (true, _) => Piece::warning(
                "nftables lists no table here and the legacy iptables backend has tables \
                 registered. The rules are in the old backend, which this build reads as a \
                 marker and does not parse. What this host filters is not visible here, and \
                 that is not the same as a host that filters nothing.",
            ),
            (false, 0) => Piece::warning(
                "No chain is on the input hook. Nothing in this ruleset decides what happens \
                 to a packet arriving at this host, so every listening socket on the ports \
                 screen is reachable by anything that can route to it.",
            ),
            (false, _) => Piece::text(
                "A chain on the input hook decides what happens to a packet no rule in it \
                 allowed. Its policy is on the row for that chain.",
            ),
        },
    );
    said.push(Piece::Blank);

    said
}

fn pf(key: &str, item: &Value) -> Vec<Piece> {
    let enabled = fields::enabled(item);
    let mut said = vec![
        Piece::title(
            "PF",
            match enabled {
                true => "enabled",
                false => "disabled",
            },
        ),
        Piece::Blank,
    ];

    for (name, value) in [
        ("rulesets", fields::tables(item).to_string()),
        (
            "anchors",
            item["anchors"].as_u64().unwrap_or_default().to_string(),
        ),
        ("on input", fields::hooked_on_input(item).to_string()),
        ("rules", fields::all_rules(item).to_string()),
        ("object", key.to_string()),
    ] {
        said.push(Piece::field(name, value));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THIS MEANS"));
    said.push(match (enabled, fields::hooked_on_input(item)) {
        (false, _) => Piece::warning(
            "pf is switched off: the rules listed here are loaded and decide nothing. The \
             Application Firewall, on its own row, may still filter by program.",
        ),
        (true, 0) => Piece::warning(
            "pf is on and no rule of its main ruleset applies to a packet arriving at this \
             Mac, so pf lets every one of them in.",
        ),
        (true, _) => Piece::text(
            "pf is on. The chains in and out of the main ruleset are its rules for each \
             direction, and their policy is what the last rule matching every packet says; \
             an anchor is a ruleset a program of this Mac fills and empties as it runs.",
        ),
    });
    said.push(Piece::Blank);

    said
}
