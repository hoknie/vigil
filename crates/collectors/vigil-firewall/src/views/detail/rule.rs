use serde_json::Value;
use vigil_view::Piece;

use super::super::fields;

const HANDLE_WIDTH: usize = 6;

const MATCHES_WIDTH: usize = 40;

const NOT_KEPT: &str = "The rules of this chain are not in the reading: it was taken by a build \
                        that did not read them, or the chain holds none.";

pub(super) fn rules(item: &Value) -> Vec<Piece> {
    let kept = fields::kept_rules(item);
    let whole = fields::all_rules(item);

    let mut said = vec![Piece::heading("WHAT THE RULES OF THIS CHAIN DO")];
    if kept.is_empty() {
        said.push(Piece::text(NOT_KEPT));
        said.push(Piece::Blank);
        return said;
    }

    said.push(Piece::Blank);
    said.push(Piece::line(format!(
        "   {:handle$}{:matches$}{}",
        "#",
        "MATCHES",
        "THEN",
        handle = HANDLE_WIDTH,
        matches = MATCHES_WIDTH
    )));
    for (handle, matches, does) in &kept {
        said.push(Piece::line(format!(
            "   {:handle$}{:matches$}{does}",
            handle,
            fitted(matches),
            handle = HANDLE_WIDTH,
            matches = MATCHES_WIDTH
        )));
    }
    said.push(Piece::Blank);

    if whole > kept.len() as u64 {
        said.push(Piece::text(format!(
            "{} of {whole} rules are listed. A chain longer than that is written by something \
             that rewrites it — fail2ban, docker, firewalld — and keeping every rule of one \
             would make this reading grow with the traffic of the host.",
            kept.len()
        )));
        said.push(Piece::Blank);
    }

    said
}

fn fitted(matches: &str) -> String {
    match matches.chars().count() >= MATCHES_WIDTH {
        false => matches.to_string(),
        true => {
            let kept: String = matches.chars().take(MATCHES_WIDTH - 2).collect();
            format!("{kept}… ")
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn drawn(pieces: &[Piece]) -> Vec<String> {
        pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Line(line) => Some(line.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_rule_is_shown_as_what_it_matches_and_what_it_does_with_what_it_matched() {
        let chain = json!({
            "rules": 2,
            "rules_kept": [
                {"handle": 10, "matches": "meta iifname lo", "does": "accept"},
                {"handle": 12, "matches": "tcp dport 22", "does": "jump allowed"}
            ]
        });

        let lines = drawn(&rules(&chain));

        assert!(
            lines[0].contains("MATCHES") && lines[0].contains("THEN"),
            "{lines:?}"
        );
        assert!(
            lines[1].contains("10") && lines[1].contains("meta iifname lo"),
            "{lines:?}"
        );
        assert!(lines[2].contains("jump allowed"), "{lines:?}");
    }

    #[test]
    fn a_chain_whose_rules_were_capped_says_how_many_of_them_are_listed() {
        let chain = json!({
            "rules": 900,
            "rules_kept": [{"handle": 1, "matches": "anything", "does": "drop"}]
        });

        let said = format!("{:?}", rules(&chain));

        assert!(said.contains("1 of 900 rules are listed"), "{said}");
    }

    #[test]
    fn a_chain_with_no_rule_in_the_reading_says_so_rather_than_drawing_an_empty_table() {
        let said = format!("{:?}", rules(&json!({"rules": 0, "rules_kept": []})));

        assert!(said.contains("not in the reading"), "{said}");
    }

    #[test]
    fn a_rule_wider_than_the_column_is_cut_so_the_two_halves_of_the_line_stay_apart() {
        let chain = json!({
            "rules": 1,
            "rules_kept": [{"handle": 1, "matches": "a".repeat(200), "does": "drop"}]
        });

        let lines = drawn(&rules(&chain));

        assert!(lines[1].chars().count() <= 80, "{:?}", lines[1]);
        assert!(lines[1].ends_with("drop"), "{:?}", lines[1]);
    }
}
