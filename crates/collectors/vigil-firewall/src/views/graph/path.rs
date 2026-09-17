use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Piece;

use super::hooks::OnTheHook;
use crate::types::Zone;
use crate::views::fields;

const HOOK_WIDTH: usize = 13;

const CHAIN_WIDTH: usize = 34;

const POLICY_WIDTH: usize = 8;

const BRANCH_AT: usize = 42;

const IN: &str = "   in ──▶ ingress ──▶ prerouting ──▶ ( routing ) ──▶ input ──▶ this host";

const ASIDE: &str = "└──▶ forward ──▶ postrouting ──▶ out";

const OUT: &str = "   out ◀── postrouting ◀── output ◀── ( routing ) ◀── a program on this host";

const NO_POLICY: &str = "—";

pub(in crate::views) fn drawn(reading: &Snapshot, key: &str, item: &Value) -> Vec<Piece> {
    let name = fields::what(key, item);

    let mut said = vec![
        Piece::title("LINK", name.clone()),
        Piece::Blank,
        Piece::field("address", fields::shown_addresses(item)),
        Piece::field("way out", the_way_out(item)),
    ];
    if let Some(zone) = zone_of(reading, &name) {
        said.push(Piece::field("zone", zone));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHERE A PACKET GOES"));
    said.push(Piece::Blank);
    said.push(Piece::line(format!("   arriving on {name}")));
    said.push(Piece::Blank);
    said.push(Piece::line(IN));
    said.push(Piece::line(format!("{}│", " ".repeat(BRANCH_AT))));
    said.push(Piece::line(format!("{}{ASIDE}", " ".repeat(BRANCH_AT))));
    said.push(Piece::Blank);
    said.push(Piece::line(OUT));
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT IS ON EACH HOOK OF THIS HOST"));
    said.push(Piece::Blank);
    for hook in OnTheHook::walked(reading) {
        said.push(Piece::line(row(&hook)));
    }
    said.push(Piece::Blank);

    said
}

fn row(hook: &OnTheHook) -> String {
    let policy = match hook.policy.is_empty() {
        true => NO_POLICY,
        false => hook.policy.as_str(),
    };

    let (policy, counted) = match hook.holds_a_chain() {
        true => (policy, format!("{} rule(s)", hook.rules)),
        false => ("", String::new()),
    };

    format!(
        "   {:hook_width$}{:chain_width$}{:policy_width$}{counted}",
        hook.hook,
        fitted(&hook.chain),
        policy,
        hook_width = HOOK_WIDTH,
        chain_width = CHAIN_WIDTH,
        policy_width = POLICY_WIDTH,
    )
    .trim_end()
    .to_string()
}

fn fitted(chain: &str) -> String {
    match chain.chars().count() > CHAIN_WIDTH {
        false => chain.to_string(),
        true => {
            let kept: String = chain.chars().take(CHAIN_WIDTH - 2).collect();
            format!("{kept}… ")
        }
    }
}

fn the_way_out(item: &Value) -> &'static str {
    match fields::the_way_out(item) {
        true => "yes — this host's default route leaves here",
        false => "no",
    }
}

fn zone_of(reading: &Snapshot, name: &str) -> Option<String> {
    let zones = Zone::of(reading);
    let named: Vec<&str> = zones
        .iter()
        .filter(|zone| zone.on.iter().any(|held| held == name))
        .map(|zone| zone.name.as_str())
        .collect();

    match named.is_empty() {
        true => None,
        false => Some(format!("{} (built by firewalld)", named.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::firewall;

    fn graph_of(name: &str) -> Vec<Piece> {
        let reading = firewall();
        let key = format!("fw-interface|{name}");
        let item = reading.items[&key].clone();

        drawn(&reading, &key, &item)
    }

    fn column_of(line: &str, wanted: char) -> usize {
        line.chars()
            .position(|drawn| drawn == wanted)
            .expect("the character is drawn on this line")
    }

    fn lines(pieces: &[Piece]) -> Vec<String> {
        pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Line(line) => Some(line.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_graph_names_the_interface_the_packet_arrives_on_and_the_address_it_answers_on() {
        let said = format!("{:?}", graph_of("eth0"));

        assert!(said.contains("eth0"), "{said}");
        assert!(said.contains("192.168.1.23"), "{said}");
        assert!(
            said.contains("default route leaves here"),
            "a reader opens this graph to find out which way out of the host is which: {said}"
        );
    }

    #[test]
    fn every_drawn_line_of_the_graph_fits_the_narrowest_terminal_this_console_supports() {
        for name in ["eth0", "lo"] {
            for line in lines(&graph_of(name)) {
                assert!(
                    line.chars().count() <= 80,
                    "{name}: {} columns — {line:?}",
                    line.chars().count()
                );
            }
        }
    }

    #[test]
    fn the_branch_below_the_routing_decision_starts_under_the_routing_decision() {
        let drawn = lines(&graph_of("eth0"));
        let path = drawn
            .iter()
            .find(|line| line.contains("( routing )"))
            .expect("the path is drawn");
        let aside = drawn
            .iter()
            .find(|line| line.contains('└'))
            .expect("the branch is drawn");

        assert!(
            column_of(path, '(') < column_of(aside, '└'),
            "a branch drawn to the left of what it branches from is an arrow pointing at \
             nothing: {path:?} then {aside:?}"
        );
        assert!(
            column_of(aside, '└') < path.chars().count(),
            "and one drawn past the right-hand end of the path it branches from is an arrow \
             pointing off the drawing: {path:?} then {aside:?}"
        );
    }

    #[test]
    fn a_hook_with_no_chain_on_it_is_drawn_with_neither_a_policy_nor_a_count() {
        let drawn = lines(&graph_of("eth0"));
        let ingress = drawn
            .iter()
            .find(|line| line.trim_start().starts_with("ingress"))
            .expect("ingress is drawn");

        assert!(ingress.contains("no chain here"), "{ingress:?}");
        assert!(
            !ingress.contains("rule(s)"),
            "a hook with no chain on it has no policy and no rules, and a zero written where \
             the policy goes reads as a chain that accepts everything: {ingress:?}"
        );
    }

    #[test]
    fn a_chain_whose_name_is_longer_than_the_column_is_cut_rather_than_pushing_the_row_over() {
        let wide = fitted(&"a".repeat(120));

        assert_eq!(wide.chars().count(), CHAIN_WIDTH);
        assert!(wide.contains('…'), "{wide}");
    }
}
