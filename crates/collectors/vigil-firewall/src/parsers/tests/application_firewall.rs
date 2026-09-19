use super::super::application_firewall::parse_application_firewall;
use crate::fixture::SOCKETFILTERFW;

#[test]
fn the_state_of_the_application_firewall_is_read_from_every_line_socketfilterfw_prints() {
    let read = parse_application_firewall(SOCKETFILTERFW).expect("reads");

    assert_eq!(read.state, Some(1));
    assert_eq!(read.enabled(), Some(true));
    assert_eq!(read.blocks_all, Some(false));
    assert_eq!(read.stealth, Some(true));
    assert_eq!(read.allows_signed, Some(true));
    assert_eq!(read.allows_downloaded_signed, Some(false));
}

#[test]
fn every_program_it_decides_for_is_read_with_its_whole_path_and_the_decision() {
    let read = parse_application_firewall(SOCKETFILTERFW).expect("reads");

    assert_eq!(
        read.applications,
        vec![
            ("/usr/sbin/cupsd".to_string(), true),
            ("/Applications/Google Chrome.app".to_string(), true),
            ("/usr/local/bin/node".to_string(), false),
        ]
    );
}

#[test]
fn the_application_firewall_of_this_kind_of_mac_switched_off_is_read_as_off() {
    let off = parse_application_firewall(
        "Firewall is disabled. (State = 0)\nFirewall has block all state set to disabled.\n\
         Firewall stealth mode is off\n",
    )
    .expect("reads");

    assert_eq!(off.enabled(), Some(false));
    assert_eq!(off.stealth, Some(false));
    assert!(off.applications.is_empty());
}

#[test]
fn output_with_no_state_in_it_is_not_an_application_firewall_that_is_off() {
    assert_eq!(parse_application_firewall(""), None);
    assert_eq!(
        parse_application_firewall("socketfilterfw: must be root\n"),
        None
    );
}
