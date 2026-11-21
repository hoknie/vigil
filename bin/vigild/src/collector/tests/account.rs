use crate::collector::rights::refused_without_root;

#[test]
fn an_account_cannot_switch_a_job_that_runs_as_root_and_nothing_is_touched() {
    let unit = crate::modules::unit_of("firewall");

    let said = refused_without_root(true, "firewall", unit).expect("refused");

    assert!(said.contains("vigil.firewall"), "{said}");
    assert!(said.contains("runs as root"), "{said}");
    assert!(said.ends_with("run this with sudo"), "{said}");
}

#[test]
fn an_account_switches_a_collector_without_a_job_in_its_own_configuration() {
    assert_eq!(refused_without_root(true, "files", None), None);
}

#[test]
fn root_and_linux_switch_every_collector_as_they_always_did() {
    assert_eq!(
        refused_without_root(false, "firewall", Some("vigil-firewall.service")),
        None
    );
}
