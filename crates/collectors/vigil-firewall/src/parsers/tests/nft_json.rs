use super::super::nft_json::{NftRefusal, parse_nft_ruleset};

const EMPTY_RULESET: &str = r#"{"nftables": [{"metainfo": {"version": "1.0.6", "release_name": "Lester Gooseberry #3", "json_schema_version": 1}}]}"#;

const ONE_TABLE: &str = r#"{"nftables": [
        {"metainfo": {"version": "1.0.6", "release_name": "Lester Gooseberry #3", "json_schema_version": 1}},
        {"table": {"family": "inet", "name": "filter", "handle": 1}},
        {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
        {"chain": {"family": "inet", "table": "filter", "name": "allowed", "handle": 2}},
        {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 4, "expr": [{"match": {"op": "==", "left": {"meta": {"key": "iifname"}}, "right": "lo"}}, {"accept": null}]}},
        {"rule": {"family": "inet", "table": "filter", "chain": "allowed", "handle": 5, "expr": [{"accept": null}]}}
    ]}"#;

#[test]
fn an_empty_file_is_a_refusal_and_not_an_empty_ruleset() {
    assert_eq!(parse_nft_ruleset(b""), Err(NftRefusal::Empty));
    assert_eq!(parse_nft_ruleset(b"\n  \n"), Err(NftRefusal::Empty));

    let host_with_no_tables = parse_nft_ruleset(EMPTY_RULESET.as_bytes()).expect("reads");
    assert!(host_with_no_tables.tables.is_empty());
    assert_eq!(host_with_no_tables.version.as_deref(), Some("1.0.6"));
}

#[test]
fn a_ruleset_in_a_shape_we_do_not_know_is_not_a_host_without_a_firewall() {
    for document in [
        r#"{"firewall": []}"#,
        r#"[{"table": {"family": "inet", "name": "filter"}}]"#,
        r#"{"nftables": {"table": {"family": "inet", "name": "filter"}}}"#,
        r#"{"nftables": [{"table": {"family": "inet", "name": "filter"}}]}"#,
    ] {
        let refusal = parse_nft_ruleset(document.as_bytes())
            .expect_err("a shape we cannot read is not a reading");
        assert!(
            matches!(refusal, NftRefusal::NotARuleset(_)),
            "{document} came back as {refusal:?}"
        );
    }
}

#[test]
fn text_that_is_not_json_comes_back_with_its_first_line_so_a_reader_knows_what_happened() {
    let refusal = parse_nft_ruleset(b"Error: syntax error, unexpected string\nnft list ruleset\n")
        .expect_err("not json");

    assert_eq!(
        refusal,
        NftRefusal::NotJson("Error: syntax error, unexpected string".into())
    );
    assert!(refusal.to_string().contains("unknown"));
}

#[test]
fn a_base_chain_carries_its_hook_priority_and_policy_and_a_regular_chain_does_not() {
    let ruleset = parse_nft_ruleset(ONE_TABLE.as_bytes()).expect("reads");

    assert_eq!(ruleset.base_chains.len(), 1);
    assert_eq!(ruleset.chains(), 2);
    let input = &ruleset.base_chains[0];
    assert_eq!(input.name, "input");
    assert_eq!(input.hook, "input");
    assert_eq!(input.policy, "drop");
    assert_eq!(input.priority, 0);
    assert_eq!(input.rules, 1);
    assert_eq!(ruleset.hooked_on_input(), 1);
}

#[test]
fn rules_are_counted_per_table_and_per_base_chain_and_never_listed_one_by_one() {
    let ruleset = parse_nft_ruleset(ONE_TABLE.as_bytes()).expect("reads");

    assert_eq!(ruleset.rules, 2);
    assert_eq!(ruleset.tables.len(), 1);
    assert_eq!(ruleset.tables[0].rules, 2);
    assert_eq!(ruleset.tables[0].chains, 2);
    assert_eq!(ruleset.families(), vec!["inet".to_string()]);
}

#[test]
fn the_handle_a_table_carries_is_read_past_rather_than_recorded() {
    let first = parse_nft_ruleset(ONE_TABLE.as_bytes()).expect("reads");
    let shuffled = ONE_TABLE.replace(
        r#"{"table": {"family": "inet", "name": "filter", "handle": 1}},"#,
        r#"{"table": {"family": "inet", "name": "filter", "handle": 97}},"#,
    );

    assert_eq!(
        first,
        parse_nft_ruleset(shuffled.as_bytes()).expect("reads")
    );
}
