use serde_json::Value;

const UNSAID: &str = "?";

const CONCATENATED: &str = " . ";

pub fn operand(value: &Value) -> String {
    match value {
        Value::Null => UNSAID.to_string(),
        Value::Bool(said) => said.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(operand)
            .collect::<Vec<String>>()
            .join(CONCATENATED),
        Value::Object(fields) => match fields.iter().next() {
            Some((name, body)) => named(name, body),
            None => UNSAID.to_string(),
        },
    }
}

fn named(name: &str, body: &Value) -> String {
    match name {
        "payload" => payload(body),
        "meta" => format!("meta {}", text(body, "key")),
        "ct" => connection(body),
        "prefix" => format!("{}/{}", operand(&body["addr"]), operand(&body["len"])),
        "range" => range(body),
        "set" => set(body),
        "elem" => operand(&body["val"]),
        "concat" => operand(body),
        "fib" => format!("fib {} {}", operand(&body["flags"]), text(body, "result")),
        "rt" => format!("rt {}", text(body, "key")),
        "&" | "|" | "^" | "<<" | ">>" => binary(name, body),
        other => format!("{other} {}", operand(body)),
    }
}

fn payload(body: &Value) -> String {
    match body.get("protocol") {
        Some(protocol) => format!("{} {}", operand(protocol), text(body, "field")),
        None => format!(
            "@{},{},{}",
            text(body, "base"),
            operand(&body["offset"]),
            operand(&body["len"])
        ),
    }
}

fn connection(body: &Value) -> String {
    match body.get("dir").and_then(Value::as_str) {
        Some(direction) => format!("ct {direction} {}", text(body, "key")),
        None => format!("ct {}", text(body, "key")),
    }
}

fn range(body: &Value) -> String {
    match body.as_array() {
        Some(ends) if ends.len() == 2 => format!("{}-{}", operand(&ends[0]), operand(&ends[1])),
        _ => operand(body),
    }
}

fn set(body: &Value) -> String {
    let Some(items) = body.as_array() else {
        return operand(body);
    };
    let listed: Vec<String> = items.iter().map(operand).collect();

    format!("{{ {} }}", listed.join(", "))
}

fn binary(operation: &str, body: &Value) -> String {
    match body.as_array() {
        Some(sides) if sides.len() == 2 => {
            format!("{} {operation} {}", operand(&sides[0]), operand(&sides[1]))
        }
        _ => format!("{operation} {}", operand(body)),
    }
}

fn text(body: &Value, field: &str) -> String {
    body.get(field)
        .and_then(Value::as_str)
        .unwrap_or(UNSAID)
        .to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_header_field_a_packet_is_matched_on_is_named_the_way_nft_names_it() {
        assert_eq!(
            operand(&json!({"payload": {"protocol": "tcp", "field": "dport"}})),
            "tcp dport"
        );
        assert_eq!(
            operand(&json!({"meta": {"key": "iifname"}})),
            "meta iifname"
        );
        assert_eq!(operand(&json!({"ct": {"key": "state"}})), "ct state");
        assert_eq!(
            operand(&json!({"ct": {"key": "state", "dir": "original"}})),
            "ct original state"
        );
    }

    #[test]
    fn an_address_with_a_prefix_a_range_and_a_set_each_read_as_one_value() {
        assert_eq!(
            operand(&json!({"prefix": {"addr": "10.0.0.0", "len": 8}})),
            "10.0.0.0/8"
        );
        assert_eq!(operand(&json!({"range": [1024, 2048]})), "1024-2048");
        assert_eq!(
            operand(&json!({"set": ["established", "related"]})),
            "{ established, related }"
        );
        assert_eq!(
            operand(&json!({"set": [{"range": [80, 88]}, 443]})),
            "{ 80-88, 443 }"
        );
    }

    #[test]
    fn a_shape_this_build_has_no_name_for_is_still_written_out_rather_than_dropped() {
        let unknown = json!({"osf": {"key": "name", "ttl": "loose"}});

        let said = operand(&unknown);

        assert!(
            said.starts_with("osf "),
            "a rule detail that silently loses half of what a rule matches on is a detail \
             that tells a reader the rule is narrower than it is: {said}"
        );
        assert_eq!(operand(&Value::Null), "?");
    }

    #[test]
    fn two_fields_matched_together_are_written_as_the_one_value_they_are() {
        assert_eq!(
            operand(&json!([{"payload": {"protocol": "ip", "field": "saddr"}}, 22])),
            "ip saddr . 22"
        );
    }
}
