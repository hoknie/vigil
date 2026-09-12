use vigil_model::{Finding, Host};

use super::Rfc5424;
use super::message::BOM;
use crate::formats::syslog::SyslogFacility;

const BUDGET: usize = 2048;

const MINIMUM_BUDGET: usize = 1536;

const _: () = assert!(BUDGET >= MINIMUM_BUDGET);

impl Rfc5424 {
    pub fn new(facility: SyslogFacility, app_name: impl Into<String>, process_id: u32) -> Self {
        Rfc5424 {
            facility,
            app_name: app_name.into(),
            process_id,
            budget: BUDGET,
        }
    }

    #[cfg(test)]
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.budget = budget.max(MINIMUM_BUDGET);
        self
    }

    pub fn line(&self, finding: &Finding, host: &Host, sent_at: &str) -> String {
        let head = self.head(finding, host, sent_at);
        let fields = self.message_fields(finding);
        let whole: usize = fields.clamped
            + fields
                .items
                .iter()
                .map(|item| item.len() + 1)
                .sum::<usize>();

        let mut kept = fields.items.len();
        loop {
            let sent: usize = fields.items[..kept]
                .iter()
                .map(|item| item.len() + 1)
                .sum::<usize>();
            let dropped = whole - sent;

            let mut message = fields.items[..kept].join(" ");
            if dropped > 0 {
                message.push_str(&format!(
                    " [vigil: truncated, {dropped} byte(s) not sent; \
                     the whole record is under event_id]"
                ));
            }

            let line = format!(
                "{head}{} {BOM}{message}",
                self.structured_data(finding, host, dropped)
            );
            if line.len() <= self.budget || kept == 1 {
                return line;
            }
            kept -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{finding, format, host};
    use super::*;
    use serde_json::json;
    use vigil_model::Evidence;

    #[test]
    fn one_finding_is_one_line_a_receiver_can_read_without_knowing_this_product() {
        let line = format().line(&finding(), &host(), "2026-09-09T12:04:05.000Z");

        assert!(
            line.starts_with(&format!("<{}>1 ", 20 * 8 + 3)),
            "facility local4, severity high: {line}"
        );
        assert!(
            line.contains(
                " 2026-09-09T12:04:02.311Z web-03.example.com vigil 4242 port.listen.new ["
            ),
            "the header names the time, the host, the app and the kind: {line}"
        );
        assert!(
            line.contains(r#"[vigil@32473 event_id="0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e1""#),
            "the identity is in the structured data: {line}"
        );
        assert!(
            line.contains("New listening socket tcp 0.0.0.0:4444"),
            "and the sentence is still there for a person: {line}"
        );
        assert!(
            line.contains(r#"port="4444""#) && line.contains(r#"cmdline="nc -l 4444""#),
            "the facts are pairs, not prose: {line}"
        );
        assert!(!line.contains('\n'), "one record is one line: {line}");
    }

    #[test]
    fn a_finding_too_long_for_syslog_says_so_instead_of_being_cut_by_the_receiver() {
        let mut huge = finding();
        huge.evidence = vec![Evidence {
            kind: "cmdline".into(),
            value: format!("java {}", "-Dsomething=very-long-value ".repeat(400)),
        }];
        huge.after = Some(json!({"blob": "x".repeat(4000)}));

        let line = format().line(&huge, &host(), "");

        assert!(
            line.len() <= BUDGET,
            "{} bytes is over the budget this sink promises",
            line.len()
        );
        assert!(
            line.contains("[vigil: truncated,"),
            "a person must see that the record is not whole: {line}"
        );
        assert!(
            line.contains(r#" truncated=""#),
            "and a parser must see it too: {line}"
        );
        assert!(
            line.contains(r#"event_id="0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e1""#)
                && line.contains(r#"finding_key="port.listen|tcp|0.0.0.0:4444""#)
                && line.contains("New listening socket"),
            "what is dropped is evidence, never the identity or the sentence: {line}"
        );
        assert!(
            line.contains("…\""),
            "an over-long value is shortened, and the shortening is visible: {line}"
        );
    }

    #[test]
    fn when_shortening_values_is_not_enough_whole_fields_go_from_the_end() {
        let mut crowded = finding();
        crowded.before = Some(json!({"protocol": "tcp", "process": {"pid": 11}}));
        crowded.evidence = (0..20)
            .map(|index| Evidence {
                kind: format!("note{index}"),
                value: "a fact about this host that is a couple of hundred bytes long ".repeat(3),
            })
            .collect();

        let line = format().line(&crowded, &host(), "");

        assert!(line.len() <= BUDGET, "{} bytes", line.len());
        assert!(
            !line.contains("after=") && !line.contains("before="),
            "the two fields nothing bounds are the first to go: {line}"
        );
        assert!(
            line.contains(r#"note0=""#),
            "and the evidence at the front survives: {line}"
        );
        assert!(
            !line.contains(r#"note19=""#),
            "while the evidence at the back does not: {line}"
        );
        assert!(line.contains("[vigil: truncated,"), "{line}");
    }

    #[test]
    fn nothing_is_announced_as_truncated_when_nothing_was() {
        let line = format().line(&finding(), &host(), "");

        assert!(line.len() <= BUDGET, "{line}");
        assert!(!line.contains("truncated"), "{line}");
        assert!(line.contains(r#"after="{"#), "{line}");
    }

    #[test]
    fn a_finding_that_is_pathological_in_every_field_still_fits_the_budget() {
        let mut monstrous = finding();
        monstrous.title = "т".repeat(4000);
        monstrous.finding_key = "\"".repeat(4000);
        monstrous.event_id = "e".repeat(4000);
        monstrous.first_seen_at = "]".repeat(4000);
        monstrous.rule = Some("r".repeat(4000));
        monstrous.redacted = vec!["/after/process/cmdline".into(); 200];
        monstrous.evidence = (0..200)
            .map(|_| Evidence {
                kind: "note".repeat(50),
                value: "v".repeat(2000),
            })
            .collect();
        let mut big_host = host();
        big_host.fqdn = Some("h".repeat(4000));
        big_host.host_id = "\\".repeat(4000);

        let line = format()
            .with_budget(MINIMUM_BUDGET)
            .line(&monstrous, &big_host, "");

        assert!(
            line.len() <= MINIMUM_BUDGET,
            "{} bytes against a floor of {MINIMUM_BUDGET}: {line}",
            line.len()
        );
        assert!(
            line.contains("[vigil: truncated,"),
            "and it still says what happened to it: {line}"
        );
    }
}
