use vigil_view::time_of_day;

use crate::ui::{Gone, Notice};

pub fn out_of_the_reading(gone: &Gone) -> Notice {
    said(
        Notice::loud(format!("{} is not in this reading any more.", gone.key)),
        gone,
    )
    .saying(
        "A finding about a flushed table or a host that stopped filtering is a finding about \
         a row that is gone by the time it is opened: that is the event itself, not a failure \
         to find it. What the reading holds now is under this.",
    )
}

pub fn not_a_row_here_yet(gone: &Gone, what: &str) -> Notice {
    said(
        Notice::loud("This console draws no row for what is waiting to send.")
            .saying(format!("The finding is about what is waiting for {what}.")),
        gone,
    )
    .saying(
        "What is waiting to be sent is counted by the agent and not yet listed object by \
         object. The receivers it sends to, and whether they are taking findings, are on this \
         screen.",
    )
}

fn said(notice: Notice, gone: &Gone) -> Notice {
    notice.saying(format!(
        "The finding said: {} ({}), and the agent last had that row in front of it at {}.",
        gone.title,
        gone.kind,
        time_of_day(&gone.last_seen)
    ))
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    fn gone() -> Gone {
        Gone::of(
            &fixture::finding("The inet table filter was deleted", Severity::High),
            "fw-table|inet filter",
        )
    }

    #[test]
    fn both_ways_a_row_can_be_missing_say_what_the_finding_said_and_when_it_said_it() {
        for notice in [
            out_of_the_reading(&gone()),
            not_a_row_here_yet(&gone(), "the ndjson receiver"),
        ] {
            let page = notice
                .lines(fixture::look(), 100)
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<String>>()
                .join(" ");

            assert!(page.contains("The inet table filter was deleted"), "{page}");
            assert!(page.contains("09:00:00"), "{page}");
        }
    }

    #[test]
    fn a_row_the_reading_lost_and_a_row_no_screen_has_are_not_told_to_a_reader_as_one_thing() {
        let lost = out_of_the_reading(&gone());
        let never = not_a_row_here_yet(&gone(), "the ndjson receiver");

        let said = |notice: &Notice| {
            notice
                .lines(fixture::look(), 78)
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<String>>()
        };

        let (lost, never) = (said(&lost), said(&never));

        assert!(lost[0].contains("not in this reading any more"), "{lost:?}");
        assert!(
            never[0].contains("draws no row for what is waiting to send"),
            "{never:?}"
        );
        assert!(
            never.iter().any(|line| line.contains("ndjson")),
            "the receiver it is about is named: {never:?}"
        );
        for line in lost.iter().chain(never.iter()) {
            assert!(
                line.chars().count() <= 78,
                "the headline of a notice is never wrapped, so it has to be short enough to \
                 be read whole at eighty columns: {line}"
            );
        }
    }
}
