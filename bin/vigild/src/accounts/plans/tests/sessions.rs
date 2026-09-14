use serde_json::json;
use vigil_model::AccountChange;
use vigil_users::fixture::users;

use super::harness::{arguments, planned};
use crate::accounts::step::Step;
use crate::accounts::utility::Utility;

#[test]
fn a_logind_session_is_ended_by_loginctl_and_one_only_utmp_saw_by_a_signal_to_its_leader() {
    assert_eq!(
        arguments(&planned(
            AccountChange::DeleteSession {
                key: "session|deploy|pts/0".into()
            },
            &users(),
            &[]
        )),
        vec![(
            Utility::Loginctl,
            ["terminate-session", "83"].map(String::from).to_vec()
        )]
    );

    let mut reading = users();
    reading.items.insert(
        "session|alice|pts/7".into(),
        json!({"user": "alice", "uid": 1002, "pid": 5150, "session_id": ""}),
    );
    assert_eq!(
        planned(
            AccountChange::DeleteSession {
                key: "session|alice|pts/7".into()
            },
            &reading,
            &[]
        ),
        vec![Step::Signal {
            pid: 5150,
            uid: Some(1002)
        }]
    );
}
