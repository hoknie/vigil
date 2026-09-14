use vigil_model::Finding;

use crate::ui::Deed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asking {
    deed: Deed,
    keys: Vec<String>,
    reason: String,
}

impl Asking {
    pub fn about(deed: Deed, reached: &[&Finding]) -> Option<Asking> {
        if reached.is_empty() {
            return None;
        }
        let mut keys: Vec<String> = Vec::new();
        for finding in reached {
            if !keys.contains(&finding.finding_key) {
                keys.push(finding.finding_key.clone());
            }
        }
        Some(Asking {
            deed,
            keys,
            reason: String::new(),
        })
    }

    pub fn deed(&self) -> Deed {
        self.deed
    }

    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    pub fn reason(&self) -> &str {
        self.reason.trim()
    }

    pub fn type_character(&mut self, character: char) {
        self.reason.push(character);
    }

    pub fn erase(&mut self) {
        self.reason.pop();
    }

    pub fn line(&self, width: usize) -> String {
        const ROOM_FOR_THE_ANSWER: usize = 8;

        let room = width.saturating_sub(1);
        let leads = [
            format!(
                " why is {} expected on this host? (Enter writes it, Esc leaves it) ",
                match self.keys.len() {
                    1 => self.keys[0].clone(),
                    many => format!("{many} object(s)"),
                }
            ),
            " why? (Enter writes it, Esc leaves it) ".to_string(),
            " reason (Enter writes it, Esc leaves it) ".to_string(),
            " reason ".to_string(),
        ];
        let lead = leads
            .iter()
            .find(|lead| lead.chars().count() + ROOM_FOR_THE_ANSWER <= room)
            .cloned()
            .unwrap_or_else(|| " ".to_string());

        let left = room.saturating_sub(lead.chars().count());
        let typed: String = match self.reason.chars().count() >= left {
            true => self
                .reason
                .chars()
                .skip(self.reason.chars().count() + 1 - left)
                .collect(),
            false => self.reason.clone(),
        };
        format!("{lead}{typed}\u{2582}")
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    fn findings() -> Vec<Finding> {
        let mut listening = fixture::finding("A new listening port", Severity::Critical);
        listening.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
        let mut same_object = fixture::finding("The owner of it changed", Severity::High);
        same_object.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
        let mut group = fixture::finding("A group gained a member", Severity::High);
        group.finding_key = "user|group|docker".into();
        vec![listening, same_object, group]
    }

    fn asking(all: &[Finding]) -> Asking {
        Asking::about(Deed::Remove, &all.iter().collect::<Vec<&Finding>>()).expect("something")
    }

    #[test]
    fn nothing_to_reach_is_nothing_to_ask_about() {
        assert_eq!(Asking::about(Deed::Remove, &[]), None);
    }

    #[test]
    fn two_findings_about_one_object_are_one_entry() {
        let all = findings();

        let asking = asking(&all);

        assert_eq!(
            asking.keys(),
            ["port.listen|tcp|0.0.0.0:4444", "user|group|docker"],
            "and the object is written down once: a file with the same entry twice is a file \
             somebody has to read twice"
        );
    }

    #[test]
    fn what_is_typed_is_the_reason_and_the_spaces_around_it_are_not() {
        let all = findings();
        let mut asking = asking(&all);

        for character in "  the staging api  ".chars() {
            asking.type_character(character);
        }

        assert_eq!(asking.reason(), "the staging api");
    }

    #[test]
    fn a_letter_taken_back_is_taken_off_the_end() {
        let all = findings();
        let mut asking = asking(&all);
        for character in "abc".chars() {
            asking.type_character(character);
        }

        asking.erase();

        assert_eq!(asking.reason(), "ab");
    }

    #[test]
    fn the_line_says_what_will_be_written_and_which_keys_end_it() {
        let all = findings();
        let asking = asking(&all);

        let line = asking.line(120);

        assert!(line.contains("2 object(s)"), "{line}");
        assert!(line.contains("Enter writes it"), "{line}");
        assert!(line.contains("Esc leaves it"), "{line}");
    }

    #[test]
    fn one_object_is_named_on_the_line_because_a_reader_is_about_to_silence_that_one() {
        let all = findings();
        let asking = Asking::about(Deed::Remove, &[&all[2]]).expect("a finding to ask about");

        assert!(asking.line(120).contains("user|group|docker"));
    }

    #[test]
    fn the_line_never_runs_off_a_narrow_terminal_and_keeps_the_end_of_what_was_typed() {
        let all = findings();
        let mut asking = asking(&all);
        for character in "a".repeat(200).chars() {
            asking.type_character(character);
        }
        asking.type_character('Z');

        for width in [40usize, 80, 120] {
            let line = asking.line(width);
            assert!(line.chars().count() <= width, "{width}: {line}");
            assert!(
                line.contains('Z'),
                "the end of what is being typed is the part a reader is looking at: {line}"
            );
        }
    }
}
