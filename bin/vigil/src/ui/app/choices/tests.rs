use vigil_model::Severity;

use super::offered::offered;
use crate::ui::Column;

#[test]
fn every_severity_and_every_column_of_the_table_is_on_the_list_of_what_can_be_narrowed() {
    let offered = offered();

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
        offered()
            .iter()
            .any(|option| option.contains("critical and above")),
        "the floor moved out of s and into f, and losing it on the way would be losing a \
         way of reading the findings"
    );
}
