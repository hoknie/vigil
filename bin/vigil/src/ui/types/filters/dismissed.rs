use std::collections::BTreeSet;

use vigil_model::Finding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dismissed {
    objects: BTreeSet<String>,
}

impl Default for Dismissed {
    fn default() -> Self {
        Dismissed::none()
    }
}

impl Dismissed {
    pub const fn none() -> Dismissed {
        Dismissed {
            objects: BTreeSet::new(),
        }
    }

    pub fn silence(&mut self, keys: impl IntoIterator<Item = String>) {
        self.objects.extend(keys);
    }

    pub fn objects(&self) -> Vec<String> {
        self.objects.iter().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn forget(&mut self, key: &str) {
        self.objects.remove(key);
    }

    pub fn bring_back(&mut self) {
        self.objects.clear();
    }

    pub fn keeping<'a>(&self, passing: Vec<&'a Finding>) -> Vec<&'a Finding> {
        match self.objects.is_empty() {
            true => passing,
            false => passing
                .into_iter()
                .filter(|finding| !self.objects.contains(&finding.finding_key))
                .collect(),
        }
    }

    pub fn settle(&mut self, findings: &[Finding]) {
        if self.objects.is_empty() {
            return;
        }
        let held: BTreeSet<&str> = findings
            .iter()
            .map(|finding| finding.finding_key.as_str())
            .collect();
        self.objects.retain(|key| held.contains(key.as_str()));
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    fn about(key: &str, title: &str) -> Finding {
        let mut finding = fixture::finding(title, Severity::High);
        finding.finding_key = key.to_string();
        finding
    }

    fn findings() -> Vec<Finding> {
        vec![
            about("port.listen|tcp|0.0.0.0:4444", "A new listening port"),
            about("port.listen|tcp|0.0.0.0:4444", "The owner of it changed"),
            about("user|group|docker", "A group gained a member"),
        ]
    }

    fn titles(findings: &[Finding], dismissed: &Dismissed) -> Vec<String> {
        dismissed
            .keeping(findings.iter().collect())
            .iter()
            .map(|finding| finding.title.clone())
            .collect()
    }

    #[test]
    fn a_console_that_silenced_nothing_shows_every_finding_it_was_given() {
        let all = findings();

        assert_eq!(titles(&all, &Dismissed::default()).len(), 3);
        assert!(Dismissed::default().is_empty());
    }

    #[test]
    fn silencing_an_object_takes_every_row_about_that_object_off_the_screen() {
        let all = findings();
        let mut dismissed = Dismissed::default();

        dismissed.silence(["port.listen|tcp|0.0.0.0:4444".to_string()]);

        assert_eq!(
            titles(&all, &dismissed),
            vec!["A group gained a member"],
            "the entry in the configuration names the object, so a row about it that stayed \
             on the screen would be a row the agent has stopped reporting"
        );
        assert_eq!(
            dismissed.objects().len(),
            1,
            "and it is one object, not two rows"
        );
    }

    #[test]
    fn a_finding_raised_about_a_silenced_object_after_the_fact_is_not_shown_either() {
        let mut all = findings();
        let mut dismissed = Dismissed::default();
        dismissed.silence(["user|group|docker".to_string()]);

        all.push(about("user|group|docker", "The same group gained another"));

        assert_eq!(titles(&all, &dismissed).len(), 2);
    }

    #[test]
    fn what_was_silenced_comes_back_because_the_entry_can_be_taken_out_again() {
        let all = findings();
        let mut dismissed = Dismissed::default();
        dismissed.silence(all.iter().map(|finding| finding.finding_key.clone()));

        dismissed.bring_back();

        assert_eq!(titles(&all, &dismissed).len(), 3);
        assert_eq!(dismissed, Dismissed::default());
    }

    #[test]
    fn the_objects_this_console_silenced_are_the_entries_it_can_take_back_out() {
        let all = findings();
        let mut dismissed = Dismissed::default();

        dismissed.silence(all.iter().map(|finding| finding.finding_key.clone()));

        assert_eq!(
            dismissed.objects(),
            ["port.listen|tcp|0.0.0.0:4444", "user|group|docker"],
            "two rows about one object are one entry in the file, so they are one entry to \
             take back out"
        );
    }

    #[test]
    fn an_object_reported_again_from_the_list_of_what_is_silenced_comes_back_on_its_own() {
        let all = findings();
        let mut dismissed = Dismissed::default();
        dismissed.silence(all.iter().map(|finding| finding.finding_key.clone()));

        dismissed.forget("user|group|docker");

        assert_eq!(dismissed.objects(), ["port.listen|tcp|0.0.0.0:4444"]);
    }

    #[test]
    fn an_object_the_agent_no_longer_holds_is_forgotten_rather_than_remembered_for_ever() {
        let all = findings();
        let mut dismissed = Dismissed::default();
        dismissed.silence(all.iter().map(|finding| finding.finding_key.clone()));

        dismissed.settle(&all[..1]);

        assert_eq!(
            dismissed.objects(),
            ["port.listen|tcp|0.0.0.0:4444"],
            "the ring drops the oldest findings, and a console that kept hiding what it no \
             longer holds would hide a new finding about that object for ever"
        );
    }
}
