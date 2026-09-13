use serde_json::Value;

use super::size::bytes;
use crate::types::Piece;

pub fn every_field(
    kind: &str,
    named: &str,
    key: &str,
    item: &Value,
    means: &[String],
) -> Vec<Piece> {
    let mut said = vec![Piece::title(kind.to_uppercase(), named), Piece::Blank];

    for (name, value) in values(item) {
        match value {
            Said::One(one) => said.push(Piece::field(name, one)),
            Said::Many(all) => {
                said.push(Piece::field(name, format!("{} in all", all.len())));
                for one in all {
                    said.push(Piece::field("", format!("· {one}")));
                }
            }
        }
    }
    said.push(Piece::field("object", key));
    said.push(Piece::Blank);

    if !means.is_empty() {
        said.push(Piece::heading("WHAT THIS IS"));
        for sentence in means {
            said.push(Piece::text(sentence.clone()));
        }
        said.push(Piece::Blank);
    }

    said
}

enum Said {
    One(String),
    Many(Vec<String>),
}

fn values(item: &Value) -> Vec<(String, Said)> {
    let Some(fields) = item.as_object() else {
        return vec![("value".to_string(), Said::One(plain(item)))];
    };

    fields
        .iter()
        .map(|(name, value)| {
            (
                name.replace('_', " "),
                match value.as_array() {
                    Some(all) => Said::Many(all.iter().map(plain).collect()),
                    None => Said::One(with_a_size(name, value)),
                },
            )
        })
        .collect()
}

fn with_a_size(name: &str, value: &Value) -> String {
    match (name.ends_with("_bytes"), value.as_u64()) {
        (true, Some(bytes)) => format!("{} ({bytes} bytes)", self::bytes(bytes)),
        _ => plain(value),
    }
}

fn plain(value: &Value) -> String {
    match value {
        Value::Null => "not read".to_string(),
        Value::String(said) => said.clone(),
        Value::Bool(yes) => match yes {
            true => "yes".to_string(),
            false => "no".to_string(),
        },
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn every_field_of_a_row_reaches_the_reader_including_the_ones_this_build_never_heard_of() {
        let item = json!({
            "image": "nginx:1.27",
            "privileged": true,
            "host_paths": ["/", "/var/run"],
            "size_bytes": 5_368_709_120u64,
            "something_new": "from a later agent",
        });

        let said = format!(
            "{:?}",
            every_field("container", "web", "container|abc", &item, &[])
        );

        assert!(said.contains("nginx:1.27"), "{said}");
        assert!(
            said.contains("host paths"),
            "the name is read, not shown raw: {said}"
        );
        assert!(said.contains("2 in all"), "{said}");
        assert!(said.contains("5.0 GB"), "a size is shown as one: {said}");
        assert!(said.contains("from a later agent"), "{said}");
        assert!(said.contains("container|abc"), "{said}");
    }

    #[test]
    fn a_field_the_agent_could_not_read_says_so_rather_than_showing_nothing() {
        let said = format!(
            "{:?}",
            every_field(
                "container",
                "web",
                "container|abc",
                &json!({"image": null}),
                &[]
            )
        );

        assert!(said.contains("not read"), "{said}");
    }
}
