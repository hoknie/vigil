use vigil_model::Severity;

use super::offered::{kind_chosen, kind_named, offered};
use crate::ui::Column;

#[test]
fn every_severity_and_every_column_of_the_table_is_on_the_list_of_what_can_be_narrowed() {
    let offered = offered(&[]);

    assert_eq!(offered.len(), Severity::KNOWN.len() + Column::ALL.len());
    assert_eq!(offered[0], "every severity");
    for severity in Severity::KNOWN.iter().skip(1) {
        assert!(
            offered
                .iter()
                .any(|option| option.contains(severity.as_str())),
            "{} is not offered",
            severity.as_str()
        );
    }
    for column in Column::ALL {
        assert!(
            offered.iter().any(|option| option.contains(column.name())),
            "{} is not offered",
            column.name()
        );
    }
}

#[test]
fn the_severity_floor_is_one_of_the_filters_and_not_a_key_of_its_own_any_more() {
    assert!(
        offered(&[])
            .iter()
            .any(|option| option.contains("critical and above")),
        "the floor moved out of s and into f, and losing it on the way would be losing a \
         way of reading the findings"
    );
}

#[test]
fn every_kind_the_agent_holds_is_offered_once_with_how_many_findings_it_would_leave() {
    let offered = offered(&[
        ("port.listen.new".to_string(), 3),
        ("user.session.new".to_string(), 1),
    ]);

    let kinds: Vec<&String> = offered
        .iter()
        .skip(Severity::KNOWN.len() + Column::ALL.len())
        .collect();
    assert_eq!(
        kinds,
        [
            "every kind",
            "only port.listen.new (3)",
            "only user.session.new (1)"
        ]
    );
}

#[test]
fn a_kind_offered_is_read_back_as_the_kind_and_no_other_option_is_taken_for_one() {
    assert_eq!(
        kind_chosen(&kind_named("port.listen.new", 12)),
        Some(Some("port.listen.new".to_string()))
    );
    assert_eq!(kind_chosen("every kind"), Some(None));
    for other in offered(&[]) {
        assert_eq!(kind_chosen(&other), None, "{other}");
    }
}
