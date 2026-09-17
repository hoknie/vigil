use serde_json::Value;

use super::operand::operand;

pub const WIDEST: usize = 160;

const ANYTHING: &str = "anything";

const NOTHING_IT_SAID: &str = "nothing this build could name";

const EQUALS: &str = "==";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NftRule {
    pub handle: i64,
    pub matches: String,
    pub does: String,
}

pub fn rule_of(body: &Value) -> NftRule {
    let mut matched: Vec<String> = Vec::new();
    let mut done: Vec<String> = Vec::new();
    let nothing: Vec<Value> = Vec::new();

    for element in body
        .get("expr")
        .and_then(Value::as_array)
        .unwrap_or(&nothing)
    {
        let Some((name, said)) = element.as_object().and_then(|fields| fields.iter().next()) else {
            continue;
        };
        match name.as_str() {
            "match" => matched.push(condition(said)),
            other => done.push(statement(other, said)),
        }
    }

    NftRule {
        handle: body.get("handle").and_then(Value::as_i64).unwrap_or(0),
        matches: shortened(match matched.is_empty() {
            true => ANYTHING.to_string(),
            false => matched.join(" "),
        }),
        does: shortened(match done.is_empty() {
            true => NOTHING_IT_SAID.to_string(),
            false => done.join(" "),
        }),
    }
}

fn condition(said: &Value) -> String {
    let left = operand(&said["left"]);
    let right = operand(&said["right"]);

    match said.get("op").and_then(Value::as_str).unwrap_or(EQUALS) {
        EQUALS => format!("{left} {right}"),
        other => format!("{left} {other} {right}"),
    }
}

fn statement(name: &str, said: &Value) -> String {
    match name {
        "accept" | "drop" | "continue" | "return" => name.to_string(),
        "reject" => reject(said),
        "jump" | "goto" => format!("{name} {}", target(said)),
        "counter" => "counter".to_string(),
        "log" => log(said),
        "limit" => limit(said),
        "masquerade" | "redirect" | "snat" | "dnat" => translation(name, said),
        "mangle" => format!("set {} {}", operand(&said["key"]), operand(&said["value"])),
        other => match said.is_null() {
            true => other.to_string(),
            false => format!("{other} {}", operand(said)),
        },
    }
}

fn reject(said: &Value) -> String {
    match said.get("type").and_then(Value::as_str) {
        Some(kind) => format!("reject with {kind}"),
        None => "reject".to_string(),
    }
}

fn target(said: &Value) -> String {
    said.get("target")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}

fn log(said: &Value) -> String {
    match said.get("prefix").and_then(Value::as_str) {
        Some(prefix) => format!("log prefix {prefix:?}"),
        None => "log".to_string(),
    }
}

fn limit(said: &Value) -> String {
    let rate = said.get("rate").map(operand).unwrap_or_default();
    let per = said.get("per").and_then(Value::as_str).unwrap_or_default();

    match (rate.is_empty(), per.is_empty()) {
        (false, false) => format!("limit rate {rate}/{per}"),
        (false, true) => format!("limit rate {rate}"),
        _ => "limit".to_string(),
    }
}

fn translation(name: &str, said: &Value) -> String {
    let address = said.get("addr").map(operand);
    let port = said.get("port").map(operand);

    match (address, port) {
        (Some(address), Some(port)) => format!("{name} to {address}:{port}"),
        (Some(address), None) => format!("{name} to {address}"),
        (None, Some(port)) => format!("{name} to port {port}"),
        (None, None) => name.to_string(),
    }
}

fn shortened(said: String) -> String {
    match said.chars().count() > WIDEST {
        false => said,
        true => {
            let kept: String = said.chars().take(WIDEST - 1).collect();
            format!("{kept}…")
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_rule_is_read_as_what_it_matches_and_what_it_then_does() {
        let rule = rule_of(&json!({
            "handle": 12,
            "expr": [
                {"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 22}},
                {"counter": {"packets": 9, "bytes": 640}},
                {"accept": null}
            ]
        }));

        assert_eq!(rule.handle, 12);
        assert_eq!(rule.matches, "tcp dport 22");
        assert_eq!(
            rule.does, "counter accept",
            "the words are the rule's, in the order the kernel walks them"
        );
    }

    #[test]
    fn the_numbers_a_counter_holds_are_left_out_of_what_a_rule_is_said_to_do() {
        let rule = rule_of(&json!({
            "handle": 1,
            "expr": [{"counter": {"packets": 4182, "bytes": 291402}}, {"accept": null}]
        }));

        assert_eq!(
            rule.does, "counter accept",
            "a rule detail carrying the two numbers of its counter would differ from the one \
             read a minute earlier on every host that passes a packet, and the detail is part \
             of the reading the differ compares"
        );
    }

    #[test]
    fn a_rule_that_matches_on_nothing_says_so_rather_than_reading_as_an_empty_cell() {
        let rule = rule_of(&json!({"handle": 3, "expr": [{"drop": null}]}));

        assert_eq!(rule.matches, ANYTHING);
        assert_eq!(rule.does, "drop");
    }

    #[test]
    fn a_rule_with_no_expression_at_all_is_still_a_rule_and_names_neither_half() {
        let rule = rule_of(&json!({"handle": 7}));

        assert_eq!(rule.matches, ANYTHING);
        assert_eq!(rule.does, NOTHING_IT_SAID);
    }

    #[test]
    fn where_a_rule_sends_a_packet_is_part_of_what_it_does() {
        let jumped = rule_of(&json!({
            "handle": 4,
            "expr": [{"jump": {"target": "allowed"}}]
        }));
        let translated = rule_of(&json!({
            "handle": 5,
            "expr": [{"dnat": {"addr": "172.17.0.2", "port": 80}}]
        }));
        let refused = rule_of(&json!({
            "handle": 6,
            "expr": [{"reject": {"type": "icmpx", "expr": "admin-prohibited"}}]
        }));

        assert_eq!(jumped.does, "jump allowed");
        assert_eq!(translated.does, "dnat to 172.17.0.2:80");
        assert_eq!(refused.does, "reject with icmpx");
    }

    #[test]
    fn a_rule_longer_than_the_widest_line_is_cut_and_says_it_was_cut() {
        let many: Vec<serde_json::Value> = (0..200)
            .map(
                |at| json!({"match": {"op": "==", "left": {"meta": {"key": "mark"}}, "right": at}}),
            )
            .collect();

        let rule = rule_of(&json!({"handle": 8, "expr": many}));

        assert_eq!(rule.matches.chars().count(), WIDEST);
        assert!(
            rule.matches.ends_with('…'),
            "a generated rule can be a kilobyte long, and a reading that keeps every one of \
             them whole is a reading nobody can afford to send twice a second: {}",
            rule.matches
        );
    }

    #[test]
    fn a_rule_that_only_counts_is_not_read_as_a_rule_that_accepts() {
        let rule = rule_of(&json!({
            "handle": 9,
            "expr": [{"log": {"prefix": "ssh "}}, {"limit": {"rate": 5, "per": "minute"}}]
        }));

        assert_eq!(rule.does, "log prefix \"ssh \" limit rate 5/minute");
    }
}
