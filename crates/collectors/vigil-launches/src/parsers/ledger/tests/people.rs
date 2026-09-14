use serde_json::json;

use super::super::snapshot::AUID_UNSET;

use super::harness::{fresh, launch, people_and_programs};

#[test]
fn a_launch_that_belongs_to_no_login_session_is_not_a_person() {
    let snapshot = fresh(&[launch(AUID_UNSET, "/usr/sbin/nginx", &["nginx"])]);

    assert!(people_and_programs(&snapshot).is_empty());
}

#[test]
fn a_login_that_etc_passwd_does_not_name_is_still_a_row_under_its_number() {
    let snapshot = fresh(&[launch(4242, "/opt/app/tool", &["tool"])]);

    assert!(snapshot.items.contains_key("run|4242|/opt/app/tool"));
    assert_eq!(
        snapshot.items["run|4242|/opt/app/tool"]["user"],
        json!(null)
    );
}
