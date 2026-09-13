pub fn family_of(finding_key: &str) -> Option<&str> {
    finding_key.split_once('|').map(|(family, _)| family)
}
