use serde_json::Value;

pub fn listed(printed: &str) -> Vec<Value> {
    let text = printed.trim();
    if text.is_empty() {
        return Vec::new();
    }

    match serde_json::from_str::<Value>(text) {
        Ok(Value::Array(items)) => items.into_iter().filter(Value::is_object).collect(),
        Ok(Value::Object(item)) => vec![Value::Object(item)],
        _ => by_line(text),
    }
}

pub fn alone(printed: &str) -> Option<Value> {
    let text = printed.trim();
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(item)) => Some(Value::Object(item)),
        Ok(Value::Array(items)) => items.into_iter().find(Value::is_object),
        _ => by_line(text).into_iter().next(),
    }
}

fn by_line(text: &str) -> Vec<Value> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(Value::is_object)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCKER: &str = "{\"ID\":\"one\"}\n{\"ID\":\"two\"}\n";

    const PODMAN: &str = "[{\"Id\":\"one\"},{\"Id\":\"two\"}]";

    #[test]
    fn one_object_a_line_and_one_array_of_objects_are_read_as_the_same_two_rows() {
        assert_eq!(listed(DOCKER).len(), 2);
        assert_eq!(listed(PODMAN).len(), 2);
        assert_eq!(listed(DOCKER)[0]["ID"], "one");
        assert_eq!(listed(PODMAN)[1]["Id"], "two");
    }

    #[test]
    fn an_engine_that_holds_nothing_prints_nothing_or_an_empty_list_and_both_are_no_rows() {
        for said in ["", "\n", "[]", "null", "   \n  \n"] {
            assert!(listed(said).is_empty(), "{said:?}");
        }
    }

    #[test]
    fn a_line_that_is_not_json_is_dropped_and_the_lines_around_it_are_still_read() {
        let mixed = "{\"ID\":\"one\"}\nWARNING: something on stdout\n{\"ID\":\"two\"}\n";

        let rows = listed(mixed);

        assert_eq!(
            rows.len(),
            2,
            "an engine that warns on standard output must not cost the reading every row printed after the warning"
        );
    }

    #[test]
    fn the_one_document_system_info_prints_is_read_whole_and_not_as_a_list_of_one() {
        let info = "{\"ServerVersion\":\"27.1.1\",\"Driver\":\"overlay2\"}";

        assert_eq!(
            alone(info).expect("one document")["ServerVersion"],
            "27.1.1"
        );
        assert_eq!(alone("").map(|_| ()), None);
    }
}
