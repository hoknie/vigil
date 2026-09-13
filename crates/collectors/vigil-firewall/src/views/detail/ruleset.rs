use serde_json::Value;
use vigil_view::Piece;

use super::super::fields;

pub(super) fn ruleset(key: &str, item: &Value) -> Vec<Piece> {
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
