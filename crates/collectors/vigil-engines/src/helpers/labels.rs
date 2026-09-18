use std::collections::BTreeMap;

use serde_json::Value;

pub const HIDDEN: &str = "[redacted]";

const SECRET_WORDS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "token",
    "apikey",
    "api_key",
    "api-key",
    "credential",
    "auth",
    "private",
    "passphrase",
];

const KEPT: usize = 64;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    pub named: BTreeMap<String, String>,
    pub redacted: bool,
    pub truncated: bool,
}

pub fn labels(item: &Value, spellings: &[&str]) -> Labels {
    for spelling in spellings {
        match item.get(spelling) {
            Some(Value::Object(said)) => return gathered(said.iter().map(written)),
            Some(Value::String(said)) if !said.is_empty() => {
                return gathered(said.split(',').filter_map(pair));
            }
            _ => continue,
        }
    }
    Labels::default()
}

impl Labels {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.named.get(name).map(String::as_str)
    }
}

fn gathered(pairs: impl Iterator<Item = (String, String)>) -> Labels {
    let mut named = BTreeMap::new();
    let mut redacted = false;
    let mut seen = 0usize;
    let mut truncated = false;

    for (name, value) in pairs {
        if name.is_empty() {
            continue;
        }
        seen += 1;
        if seen > KEPT {
            truncated = true;
            continue;
        }
        match secret(&name) {
            true => {
                named.insert(name, HIDDEN.to_string());
                redacted = true;
            }
            false => {
                named.insert(name, value);
            }
        }
    }

    Labels {
        named,
        redacted,
        truncated,
    }
}

fn written((name, value): (&String, &Value)) -> (String, String) {
    let said = match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    (name.trim().to_string(), said)
}

fn pair(written: &str) -> Option<(String, String)> {
    let (name, value) = written.split_once('=')?;
    Some((name.trim().to_string(), value.trim().to_string()))
}

fn secret(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SECRET_WORDS.iter().any(|word| lower.contains(word))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn labels_printed_as_one_line_read_as_the_same_map_as_labels_printed_as_an_object() {
        let docker = json!({"Labels": "com.docker.compose.project=shop,role=web"});
        let podman = json!({"Labels": {"com.docker.compose.project": "shop", "role": "web"}});

        let one = labels(&docker, &["Labels"]);
        let other = labels(&podman, &["Labels"]);

        assert_eq!(one.named, other.named);
        assert_eq!(one.get("com.docker.compose.project"), Some("shop"));
    }

    #[test]
    fn a_label_whose_name_says_it_holds_a_secret_never_reaches_the_reading_with_its_value_in_it() {
        let item = json!({"Labels": {
            "role": "web",
            "vault.token": "s.9a7f6e5d4c3b",
            "DB_PASSWORD": "hunter2",
            "com.corp.api-key": "AKIA1234",
        }});

        let read = labels(&item, &["Labels"]);

        assert!(
            read.redacted,
            "the row must say that something was hidden, or a reader sees an empty value and believes the label is empty"
        );
        for name in ["vault.token", "DB_PASSWORD", "com.corp.api-key"] {
            assert_eq!(read.get(name), Some(HIDDEN), "{name} kept its value");
        }
        assert_eq!(read.get("role"), Some("web"));
        let written = serde_json::to_string(&read.named).expect("plain data");
        for leaked in ["hunter2", "s.9a7f6e5d4c3b", "AKIA1234"] {
            assert!(
                !written.contains(leaked),
                "{leaked} reached the reading, and what reached the buffer has already left \
                 this host: {written}"
            );
        }
    }

    #[test]
    fn a_container_labelled_by_the_hundred_keeps_a_bounded_number_of_them_and_says_it_did() {
        let many: BTreeMap<String, String> = (0..200)
            .map(|index| (format!("label{index:03}"), "x".to_string()))
            .collect();
        let item = json!({"Labels": many});

        let read = labels(&item, &["Labels"]);

        assert_eq!(read.named.len(), KEPT);
        assert!(
            read.truncated,
            "a row that quietly drops what it could not hold is a row whose absence of a \
             label means nothing"
        );
    }

    #[test]
    fn a_host_that_labels_nothing_gives_an_empty_map_and_hides_nothing() {
        for item in [json!({}), json!({"Labels": ""}), json!({"Labels": null})] {
            let read = labels(&item, &["Labels"]);

            assert!(read.named.is_empty(), "{item}");
            assert!(!read.redacted, "{item}");
        }
    }
}
