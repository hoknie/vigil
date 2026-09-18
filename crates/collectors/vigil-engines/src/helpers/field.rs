use serde_json::Value;

pub fn at<'a>(item: &'a Value, spelling: &str) -> Option<&'a Value> {
    let mut walked = item;
    for step in spelling.split('.') {
        walked = walked.get(step)?;
    }
    Some(walked)
}

pub fn said(read: Option<String>) -> Value {
    match read {
        Some(text) => Value::String(text),
        None => Value::Null,
    }
}

pub fn text(item: &Value, spellings: &[&str]) -> Option<String> {
    for spelling in spellings {
        match at(item, spelling) {
            Some(Value::String(said)) if !said.is_empty() => return Some(said.clone()),
            Some(Value::Number(said)) => return Some(said.to_string()),
            _ => continue,
        }
    }
    None
}

pub fn flag(item: &Value, spellings: &[&str]) -> Option<bool> {
    for spelling in spellings {
        match at(item, spelling) {
            Some(Value::Bool(said)) => return Some(*said),
            Some(Value::String(said)) if said.eq_ignore_ascii_case("true") => return Some(true),
            Some(Value::String(said)) if said.eq_ignore_ascii_case("false") => return Some(false),
            _ => continue,
        }
    }
    None
}

pub fn number(item: &Value, spellings: &[&str]) -> Option<u64> {
    for spelling in spellings {
        match at(item, spelling) {
            Some(Value::Number(said)) => return said.as_u64(),
            Some(Value::String(said)) => {
                if let Ok(read) = said.parse::<u64>() {
                    return Some(read);
                }
            }
            _ => continue,
        }
    }
    None
}

pub fn words(item: &Value, spellings: &[&str]) -> Vec<String> {
    for spelling in spellings {
        match at(item, spelling) {
            Some(Value::Array(said)) => {
                let mut read: Vec<String> = said
                    .iter()
                    .filter_map(|one| one.as_str().map(str::to_string))
                    .filter(|one| !one.is_empty())
                    .collect();
                read.sort_unstable();
                read.dedup();
                return read;
            }
            Some(Value::String(said)) if !said.is_empty() => {
                let mut read: Vec<String> = said
                    .split(',')
                    .map(str::trim)
                    .filter(|one| !one.is_empty())
                    .map(str::to_string)
                    .collect();
                read.sort_unstable();
                read.dedup();
                return read;
            }
            _ => continue,
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_same_value_is_found_under_the_name_either_engine_prints_it_by() {
        let docker = json!({"ID": "18ad9bdc4c87", "Internal": "false", "Size": "1.71GB"});
        let podman = json!({"id": "18ad9bdc4c87", "internal": false, "Size": 1_836_000_000u64});

        assert_eq!(
            text(&docker, &["ID", "Id", "id"]).as_deref(),
            Some("18ad9bdc4c87")
        );
        assert_eq!(
            text(&podman, &["ID", "Id", "id"]).as_deref(),
            Some("18ad9bdc4c87")
        );
        assert_eq!(flag(&docker, &["Internal", "internal"]), Some(false));
        assert_eq!(flag(&podman, &["Internal", "internal"]), Some(false));
        assert_eq!(number(&podman, &["Size"]), Some(1_836_000_000));
    }

    #[test]
    fn a_field_the_engine_did_not_print_is_written_as_null_and_never_left_out_of_the_row() {
        assert_eq!(said(None), Value::Null);
        assert_eq!(said(Some("bridge".into())), json!("bridge"));
    }

    #[test]
    fn a_value_one_engine_nests_and_the_other_writes_flat_is_read_by_the_path_that_finds_it() {
        let podman = json!({"host": {"cgroupVersion": "v2", "security": {"rootless": true}}});

        assert_eq!(
            text(&podman, &["CgroupVersion", "host.cgroupVersion"]).as_deref(),
            Some("v2")
        );
        assert_eq!(flag(&podman, &["host.security.rootless"]), Some(true));
        assert_eq!(text(&podman, &["host.security.nothing.of.the.sort"]), None);
    }

    #[test]
    fn a_field_that_is_not_there_is_nothing_rather_than_an_empty_string() {
        let item = json!({"ID": "", "Digest": "<none>"});

        assert_eq!(
            text(&item, &["ID"]),
            None,
            "an engine prints an empty string where it has nothing, and a row carrying one says the field exists and is blank"
        );
        assert_eq!(text(&item, &["Tag"]), None);
        assert!(words(&item, &["Networks"]).is_empty());
    }

    #[test]
    fn a_list_printed_as_one_comma_separated_line_reads_as_the_same_list_as_an_array() {
        let docker = json!({"Networks": "shop_default,bridge"});
        let podman = json!({"Networks": ["bridge", "shop_default"]});

        assert_eq!(words(&docker, &["Networks"]), words(&podman, &["Networks"]));
        assert_eq!(
            words(&docker, &["Networks"]),
            vec!["bridge", "shop_default"]
        );
    }

    #[test]
    fn a_list_is_sorted_and_deduplicated_so_that_the_order_an_engine_prints_in_is_not_a_change() {
        let first = json!({"Mounts": ["/srv/www", "/etc", "/srv/www"]});
        let again = json!({"Mounts": ["/etc", "/srv/www"]});

        assert_eq!(
            words(&first, &["Mounts"]),
            words(&again, &["Mounts"]),
            "docker and podman print a container's mounts in the order they were declared, \
             and a compose file edited elsewhere would be a change in every row of the project"
        );
    }
}
