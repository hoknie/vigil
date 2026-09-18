use serde::Deserialize;

use crate::types::suppression::Suppression;

#[derive(Debug, Deserialize)]
struct Held {
    #[serde(default)]
    suppressions: Vec<Suppression>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Apart {
    #[serde(default)]
    suppressions: Vec<Suppression>,
}

pub fn silenced(text: &str) -> Result<Vec<Suppression>, String> {
    let held: Held = serde_yaml::from_str(text).map_err(|error| error.to_string())?;

    for (index, suppression) in held.suppressions.iter().enumerate() {
        suppression
            .validate()
            .map_err(|cause| format!("suppression #{}: {cause}", index + 1))?;
    }
    Ok(held.suppressions)
}

pub fn apart(text: &str) -> Result<Vec<Suppression>, String> {
    let document: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    if document.is_null() {
        return Ok(Vec::new());
    }
    let held: Apart = serde_yaml::from_value(document).map_err(|error| error.to_string())?;

    for (index, suppression) in held.suppressions.iter().enumerate() {
        suppression
            .validate()
            .map_err(|cause| format!("suppression #{}: {cause}", index + 1))?;
    }
    Ok(held.suppressions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_of_its_own_with_nothing_in_it_yet_silences_nothing_and_is_not_an_error() {
        for text in [
            "",
            "\n",
            "# written by hand, nothing yet\n",
            "suppressions: []\n",
        ] {
            assert!(apart(text).expect("reads").is_empty(), "{text:?}");
        }
    }

    #[test]
    fn a_file_of_its_own_holds_suppressions_and_nothing_else() {
        let refused = apart("suppressions: []\nreporters: []\n").expect_err("must not be accepted");

        assert!(
            refused.contains("reporters"),
            "a key the daemon reads nowhere in a file of suppressions is a key somebody believes \
             does something: {refused}"
        );
    }

    #[test]
    fn an_entry_in_a_file_of_its_own_is_held_to_the_same_rules_as_one_in_the_configuration() {
        let refused = apart("suppressions:\n  - finding_key: \"a|b\"\n").expect_err("no reason");

        assert!(refused.contains("suppression #1"), "{refused}");
        assert!(refused.contains("reason"), "{refused}");
    }

    #[test]
    fn a_file_with_nothing_written_down_holds_no_suppressions_and_is_not_an_error() {
        for text in ["state_dir: /var/lib/vigil\n", "suppressions: []\n"] {
            assert!(silenced(text).expect("reads").is_empty(), "{text}");
        }
    }

    #[test]
    fn an_entry_that_would_stop_the_daemon_at_the_door_is_refused_here_first() {
        let refused = silenced("suppressions:\n  - reason: nothing to silence\n")
            .expect_err("must not be accepted");

        assert!(refused.contains("suppression #1"), "{refused}");
        assert!(refused.contains("silences nothing"), "{refused}");
    }

    #[test]
    fn a_file_that_is_not_yaml_at_all_says_so_rather_than_reading_as_empty() {
        assert!(silenced("suppressions:\n  - - - :\n").is_err());
    }

    #[test]
    fn every_other_key_in_the_file_is_none_of_this_crates_business() {
        let held = silenced(
            "state_dir: /var/lib/vigil\nreporters: []\nsuppressions:\n  - finding_key: \"a|b\"\n    reason: ours\n",
        )
        .expect("reads");

        assert_eq!(held.len(), 1);
        assert_eq!(held[0].reason, "ours");
    }
}
