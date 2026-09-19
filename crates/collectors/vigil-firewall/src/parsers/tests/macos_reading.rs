use vigil_model::class_of;

use super::super::{
    APPLICATION_FIREWALL_ROW, MacosFirewallReading, PF_SUMMARY, Pf, macos_firewall_snapshot,
    understood,
};
use crate::fixture::{firewall_on_macos, pf_dump, unprivileged_dump};

#[test]
fn pf_on_a_mac_is_read_as_a_ruleset_whose_main_part_is_a_table_and_whose_anchors_are_tables_too() {
    let read = firewall_on_macos();

    assert_eq!(read.items[PF_SUMMARY]["enabled"], true);
    assert_eq!(read.items[PF_SUMMARY]["hooked_on_input"], 1);
    assert_eq!(read.items[PF_SUMMARY]["anchors"], 4);
    assert_eq!(read.items["fw-table|pf main"]["anchor"], false);
    assert_eq!(
        read.items["fw-table|pf com.apple/200.AirDrop/Bonjour"]["rules"],
        2
    );
    assert_eq!(
        read.items["fw-table|pf com.apple/200.AirDrop/Bonjour"]["anchor"],
        true
    );
    assert_eq!(read.items["fw-chain|pf main|in"]["policy"], "drop");
    assert_eq!(read.items["fw-chain|pf main|in"]["hook"], "input");
    assert_eq!(read.items["fw-chain|pf main|out"]["policy"], "accept");
    assert_eq!(read.items["fw-chain|pf main|rdr"]["hook"], "prerouting");
    assert_eq!(read.items["fw-chain|pf main|nat"]["hook"], "postrouting");
}

#[test]
fn a_mac_where_pf_is_off_has_nothing_on_the_input_hook_whatever_rules_are_loaded() {
    let understood = understood(&pf_dump(false));
    let pf = understood.pf.expect("reads");
    let read = macos_firewall_snapshot(
        "2026-09-19T09:00:00.000Z",
        &MacosFirewallReading {
            pf: Some(Pf {
                enabled: pf.enabled,
                main: &pf.main,
                anchors: &pf.anchors,
            }),
            application_firewall: None,
            interfaces: &[],
            counting: false,
        },
    );

    assert_eq!(read.items[PF_SUMMARY]["enabled"], false);
    assert_eq!(
        read.items[PF_SUMMARY]["hooked_on_input"], 0,
        "rules loaded into a pf that is off decide nothing"
    );
    assert_eq!(
        read.items["fw-chain|pf main|in"]["policy"], "drop",
        "the policy is what the rules say, whether or not pf is on to apply them"
    );
}

#[test]
fn the_application_firewall_is_a_row_of_its_own_with_the_programs_it_decides_for() {
    let read = firewall_on_macos();
    let row = &read.items[APPLICATION_FIREWALL_ROW];

    assert_eq!(row["enabled"], true);
    assert_eq!(row["stealth"], true);
    assert_eq!(row["applications_allowed"], 2);
    assert_eq!(row["applications_blocked"], 1);
    assert_eq!(
        row["applications"][1]["path"],
        "/Applications/Google Chrome.app"
    );
}

#[test]
fn a_counter_a_mac_does_not_keep_is_left_out_of_the_row_rather_than_written_as_zero() {
    let read = firewall_on_macos();
    let en0 = &read.items["fw-interface|en0"];

    assert_eq!(en0["dropped_in"], 7);
    assert!(
        en0.get("dropped_out").is_none(),
        "macOS counts no packets dropped on the way out, and a zero reads as none dropped: {en0}"
    );
}

#[test]
fn a_dump_taken_without_root_reads_the_application_firewall_and_says_pf_was_not_read() {
    let understood = understood(&unprivileged_dump());

    assert!(understood.application_firewall.is_ok());
    let refusal = understood.pf.expect_err("pfctl answers root alone");
    assert!(refusal.contains("Permission denied"), "{refusal}");
}

#[test]
fn every_row_of_a_mac_is_a_class_the_rules_and_the_screen_already_know() {
    let read = firewall_on_macos();
    let mut classes: Vec<&str> = read.items.keys().map(|key| class_of(key)).collect();
    classes.sort_unstable();
    classes.dedup();

    assert_eq!(
        classes,
        vec![
            "fw-application",
            "fw-chain",
            "fw-interface",
            "fw-summary",
            "fw-table"
        ]
    );
}
