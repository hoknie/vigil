pub const IP_TABLES_NAMES: &str = "/proc/net/ip_tables_names";

pub const IP6_TABLES_NAMES: &str = "/proc/net/ip6_tables_names";

pub fn parse_ip_tables_names(text: &str) -> Vec<String> {
    let mut names: Vec<String> = text
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .collect();

    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_held_by_the_legacy_backend_are_degraded_and_not_absent() {
        let registered = parse_ip_tables_names("filter\nnat\nmangle\n");

        assert_eq!(registered, vec!["filter", "mangle", "nat"]);
        assert!(
            !registered.is_empty(),
            "a host whose nft ruleset is empty and whose legacy tables are registered is a host \
             this build cannot read, not a host without a firewall: the collector answers \
             Health::Degraded over this list and never a finding that the firewall is off"
        );
    }

    #[test]
    fn a_host_where_the_legacy_backend_registered_nothing_leaves_no_marker() {
        assert!(parse_ip_tables_names("").is_empty());
        assert!(parse_ip_tables_names("\n  \n").is_empty());
    }

    #[test]
    fn the_same_table_named_twice_is_one_table() {
        assert_eq!(parse_ip_tables_names("filter\nfilter\n"), vec!["filter"]);
    }
}
