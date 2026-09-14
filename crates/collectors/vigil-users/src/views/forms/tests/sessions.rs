use serde_json::json;
use vigil_model::{AccountChange, Changing};

use super::harness::change;
use crate::fixture::users;

#[test]
fn a_session_is_ended_by_its_row_and_a_row_about_login_records_is_not_a_session() {
    let reading = users().with(
        "session|ghost|pts/9",
        json!({"user": "ghost", "session_id": "", "pid": 0}),
    );

    assert_eq!(
        change(
            "logged in",
            &reading,
            Some("session|deploy|pts/0"),
            Changing::Delete,
            None
        ),
        Ok(AccountChange::DeleteSession {
            key: "session|deploy|pts/0".into()
        })
    );
    for (at, why) in [
        ("session-source|logind", "not a session"),
        ("session|ghost|pts/9", "nothing to end it by"),
    ] {
        let said =
            change("logged in", &reading, Some(at), Changing::Delete, None).expect_err("refused");
        assert!(said.contains(why), "{at}: {said}");
    }
}
