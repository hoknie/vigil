use serde_json::Value;

use crate::ui::types::content::change::{ChangeLine, Mark};

pub fn lines(before: Option<&Value>, after: Option<&Value>) -> Vec<ChangeLine> {
    let before = flatten(before);
    let after = flatten(after);

    let mut paths: Vec<&String> = before.keys().chain(after.keys()).collect();
    paths.sort_unstable();
    paths.dedup();

    let mut out = Vec::new();
    for path in paths {
        match (before.get(path), after.get(path)) {
            (Some(old), Some(new)) if old == new => out.push(ChangeLine {
                mark: Mark::Same,
                path: path.clone(),
                value: new.clone(),
            }),
            (Some(old), Some(new)) => {
                out.push(ChangeLine {
                    mark: Mark::Removed,
                    path: path.clone(),
                    value: old.clone(),
                });
                out.push(ChangeLine {
                    mark: Mark::Added,
                    path: path.clone(),
                    value: new.clone(),
                });
            }
            (Some(old), None) => out.push(ChangeLine {
                mark: Mark::Removed,
                path: path.clone(),
                value: old.clone(),
            }),
            (None, Some(new)) => out.push(ChangeLine {
                mark: Mark::Added,
                path: path.clone(),
                value: new.clone(),
            }),
            (None, None) => {}
        }
    }

    out
}

fn flatten(value: Option<&Value>) -> std::collections::BTreeMap<String, String> {
    let mut flat = std::collections::BTreeMap::new();
    if let Some(value) = value {
        walk(String::new(), value, &mut flat);
    }
    flat
}

fn walk(path: String, value: &Value, flat: &mut std::collections::BTreeMap<String, String>) {
    match value {
        Value::Object(fields) if !fields.is_empty() => {
            for (name, field) in fields {
                let child = match path.is_empty() {
                    true => name.clone(),
                    false => format!("{path}.{name}"),
                };
                walk(child, field, flat);
            }
        }
        Value::String(text) => {
            flat.insert(path, text.clone());
        }
        Value::Null => {
            flat.insert(path, "null".to_string());
        }
        other => {
            flat.insert(path, other.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn socket(executable: &str, user: &str) -> Value {
        json!({
            "protocol": "tcp",
            "address": "0.0.0.0",
            "port": 4444,
            "user": user,
            "process": {"exe": executable, "exe_deleted": false},
        })
    }

    #[test]
    fn a_field_that_changed_shows_what_it_was_and_what_it_is() {
        let changed = lines(
            Some(&socket("/usr/sbin/nginx", "root")),
            Some(&socket("/tmp/.x/nc", "www-data")),
        );

        let executable: Vec<&ChangeLine> = changed
            .iter()
            .filter(|line| line.path == "process.exe")
            .collect();
        assert_eq!(executable.len(), 2);
        assert_eq!(executable[0].mark, Mark::Removed);
        assert_eq!(executable[0].value, "/usr/sbin/nginx");
        assert_eq!(executable[1].mark, Mark::Added);
        assert_eq!(executable[1].value, "/tmp/.x/nc");
    }

    #[test]
    fn what_did_not_change_is_still_shown_so_the_reader_sees_the_whole_object() {
        let changed = lines(
            Some(&socket("/usr/sbin/nginx", "root")),
            Some(&socket("/usr/sbin/nginx", "www-data")),
        );

        let port = changed
            .iter()
            .find(|line| line.path == "port")
            .expect("the port is part of the object");
        assert_eq!(port.mark, Mark::Same);
        assert_eq!(port.value, "4444");
    }

    #[test]
    fn a_new_object_is_all_additions_and_a_removed_one_all_removals() {
        let appeared = lines(None, Some(&socket("/tmp/.x/nc", "www-data")));
        assert!(appeared.iter().all(|line| line.mark == Mark::Added));
        assert!(appeared.iter().any(|line| line.path == "process.exe"));

        let gone = lines(Some(&socket("/tmp/.x/nc", "www-data")), None);
        assert!(gone.iter().all(|line| line.mark == Mark::Removed));
    }

    #[test]
    fn a_field_that_is_present_and_empty_is_not_the_same_as_a_missing_one() {
        let changed = lines(
            Some(&json!({"process": null})),
            Some(&json!({"process": {"exe": "/bin/nc"}})),
        );

        assert_eq!(changed[0].mark, Mark::Removed);
        assert_eq!(changed[0].value, "null");
        assert_eq!(changed[1].path, "process.exe");
    }
}
