use super::super::auditd::log_file_of;

#[test]
fn the_audit_log_is_read_where_auditd_conf_says_auditd_writes_it() {
    let moved = "\
#
# This file controls the configuration of the audit daemon
#
local_events = yes
write_logs = yes
log_file = /srv/audit/audit.log
log_group = root
";

    assert_eq!(
        log_file_of(moved).as_deref(),
        Some("/srv/audit/audit.log"),
        "a host whose auditd writes elsewhere is a host where every launch passes unseen \
         while the health line blames a stopped auditd"
    );
}

#[test]
fn an_auditd_conf_that_names_no_log_file_leaves_the_place_every_distribution_ships() {
    assert_eq!(
        log_file_of("write_logs = yes\n# log_file = /x/y.log\n"),
        None
    );
    assert_eq!(log_file_of("log_file = audit.log\n"), None);
    assert_eq!(log_file_of(""), None);
}
