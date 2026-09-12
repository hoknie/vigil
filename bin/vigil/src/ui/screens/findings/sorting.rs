use vigil_model::{Finding, Severity};

use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[AS_READ, "TIME", "SEVERITY", "KIND", "TITLE", "OBJECT"];

pub fn sort(passing: &mut [&Finding], sorting: Sorting) {
    if sorting.as_read() {
        return;
    }
    passing.sort_by(|left, right| {
        let ordering = key(left, sorting.by).cmp(&key(right, sorting.by));
        match sorting.descending {
            true => ordering.reverse(),
            false => ordering,
        }
    });
}

fn key(finding: &Finding, by: usize) -> String {
    match by {
        1 => finding.observed_at.to_string(),
        2 => format!("{}{}", rank(&finding.severity), finding.severity.as_str()),
        3 => finding.kind.as_str().to_lowercase(),
        4 => finding.title.to_lowercase(),
        5 => finding.finding_key.to_lowercase(),
        _ => String::new(),
    }
}

fn rank(severity: &Severity) -> u8 {
    match severity {
        Severity::Info => 0,
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
        Severity::Unknown(_) => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;

    fn findings() -> Vec<Finding> {
        let mut quiet = fixture::finding("b quiet one", Severity::Low);
        quiet.observed_at = "2026-09-09T09:00:03.000Z".into();
        quiet.finding_key = "z|last".into();
        let mut loud = fixture::finding("a loud one", Severity::Critical);
        loud.observed_at = "2026-09-09T09:00:01.000Z".into();
        loud.finding_key = "a|first".into();
        let mut middling = fixture::finding("c middling one", Severity::Medium);
        middling.observed_at = "2026-09-09T09:00:02.000Z".into();
        middling.finding_key = "m|middle".into();
        middling.kind = vigil_model::Kind::from("user.account.new".to_string());
        quiet.kind = vigil_model::Kind::from("exec.from_writable_path".to_string());
        vec![quiet, loud, middling]
    }

    fn titles(all: &[Finding], sorting: Sorting) -> Vec<String> {
        let mut passing: Vec<&Finding> = all.iter().collect();
        sort(&mut passing, sorting);
        passing
            .iter()
            .map(|finding| finding.title.clone())
            .collect()
    }

    #[test]
    fn the_order_a_console_nobody_sorted_shows_is_the_order_the_agent_sent() {
        let all = findings();

        assert_eq!(
            titles(&all, Sorting::default()),
            vec!["b quiet one", "a loud one", "c middling one"],
            "a script reads this console, and its answer must not change because a key was \
             added to the interactive one"
        );
    }

    #[test]
    fn a_column_sorts_both_ways_round_and_the_other_way_is_the_reverse() {
        let all = findings();

        let up = titles(&all, Sorting::of(1));
        let down = titles(&all, Sorting::of(2));

        assert_eq!(up, vec!["a loud one", "c middling one", "b quiet one"]);
        assert_eq!(down.into_iter().rev().collect::<Vec<String>>(), up);
    }

    #[test]
    fn severity_sorts_by_what_it_means_and_not_by_how_it_is_spelled() {
        let all = findings();

        let by_severity = titles(&all, Sorting::of(3));

        assert_eq!(
            by_severity,
            vec!["b quiet one", "c middling one", "a loud one"],
            "alphabetically critical would come before low, and a reader looking for the \
             worst would be shown the quietest"
        );
    }

    #[test]
    fn two_rows_that_sort_the_same_keep_the_order_they_arrived_in() {
        let mut all = findings();
        for finding in all.iter_mut() {
            finding.severity = Severity::Low;
        }

        let order = titles(&all, Sorting::of(3));

        assert_eq!(
            order,
            vec!["b quiet one", "a loud one", "c middling one"],
            "the list is redrawn on every answer from the daemon, and rows that tie must not \
             shuffle between two readings of the same host"
        );
    }

    #[test]
    fn every_column_the_table_draws_can_be_sorted_by_its_own_heading() {
        let all = findings();

        for (at, column) in SORTED_BY.iter().enumerate().skip(1) {
            let sorted = titles(
                &all,
                Sorting {
                    by: at,
                    descending: false,
                },
            );
            assert_eq!(sorted.len(), 3, "{column}");
            assert_ne!(
                key(&all[0], at),
                key(&all[1], at),
                "{column} reads the same for two different findings, so sorting by it does \
                 nothing a reader can see"
            );
        }
    }
}
