use serde_json::Value;

pub fn text<'a>(item: &'a Value, field: &str) -> Option<&'a str> {
    item.get(field).and_then(Value::as_str)
}

pub fn flag(item: &Value, field: &str) -> bool {
    item.get(field).and_then(Value::as_bool) == Some(true)
}

pub fn number(item: &Value, field: &str) -> Option<u64> {
    item.get(field).and_then(Value::as_u64)
}

pub fn strings<'a>(item: &'a Value, field: &str) -> Vec<&'a str> {
    item.get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .collect()
}

pub fn marked(key: &str) -> bool {
    !key.starts_with("exec|") && !key.starts_with("run|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_that_says_the_reading_stopped_growing_is_not_counted_as_a_program_that_was_run() {
        assert!(marked("launches|capped"));
        assert!(marked("launches|dropping"));
        assert!(marked("launches|source"));
        assert!(marked("processes|unresolved"));
        assert!(!marked("run|alice|/usr/bin/nc"));
        assert!(!marked("exec|/usr/sbin/nginx|root"));
    }
}
