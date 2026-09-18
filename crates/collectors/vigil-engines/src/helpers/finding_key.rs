pub const FAMILY: &str = "engine";

pub fn finding_key(key: &str) -> String {
    format!("{FAMILY}|{key}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_finding_about_a_row_is_keyed_by_the_family_and_then_the_row_itself() {
        assert_eq!(
            finding_key("docker|image|sha256:18ad9bdc4c87"),
            "engine|docker|image|sha256:18ad9bdc4c87",
            "the console cuts the family off and lands on the reading's own key, so the row a \
             finding is about is found without a table that maps one to the other"
        );
    }
}
