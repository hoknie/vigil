use vigil_model::Change;
use vigil_rules::programs_for;

use super::super::set::listening_port_rules;
use crate::fixture;

#[test]
fn every_finding_about_a_socket_names_the_program_a_suppression_can_be_written_for() {
    let nginx = fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root");
    let netcat = fixture::socket("0.0.0.0", 443, "/tmp/.x/nc", "www-data");
    let cases = vec![
        (
            "a service was deployed",
            Change::Added {
                key: "tcp|0.0.0.0:443".into(),
                after: nginx.clone(),
            },
            "/usr/sbin/nginx",
        ),
        (
            "a service went away",
            Change::Removed {
                key: "tcp|0.0.0.0:443".into(),
                before: nginx.clone(),
            },
            "/usr/sbin/nginx",
        ),
        (
            "another program took the port over",
            Change::Changed {
                key: "tcp|0.0.0.0:443".into(),
                before: nginx,
                after: netcat,
            },
            "/tmp/.x/nc",
        ),
    ];

    for (what, change, program) in cases {
        let fired = programs_for(listening_port_rules(), std::slice::from_ref(&change));
        assert!(!fired.is_empty(), "{what} fired nothing");
        for (kind, named) in fired {
            assert_eq!(named.as_deref(), Some(program), "{what}: {kind}");
        }
    }
}
