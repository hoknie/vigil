use vigil_model::{Finding, Host, State};

use super::Rfc5424;
use super::text::sd_value;

const SD_ID: &str = "vigil@32473";

impl Rfc5424 {
    pub(super) fn structured_data(&self, finding: &Finding, host: &Host, dropped: usize) -> String {
        let mut block = String::from("[");
        block.push_str(SD_ID);

        let mut put = |name: &str, value: &str| {
            block.push_str(&format!(" {name}=\"{}\"", sd_value(value)));
        };

        put("event_id", &finding.event_id);
        put("finding_key", &finding.finding_key);
        put("kind", finding.kind.as_str());
        put("severity", finding.severity.as_str());
        put(
            "state",
            match finding.state {
                State::Open => "open",
                State::Resolved => "resolved",
            },
        );
        put("occurrences", &finding.occurrences.to_string());
        put("first_seen", &finding.first_seen_at);
        put("host_id", &host.host_id);
        if let Some(rule) = &finding.rule {
            put("rule", rule);
        }
        if dropped > 0 {
            put("truncated", &dropped.to_string());
        }

        block.push(']');
        block
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{finding, format, host};

    #[test]
    fn a_quote_or_a_bracket_in_a_value_still_leaves_the_structured_data_parseable() {
        let mut awkward = finding();
        awkward.finding_key = r#"port.listen|tcp|"]\evil"#.into();

        let line = format().line(&awkward, &host(), "");

        assert!(
            line.contains(r#"finding_key="port.listen|tcp|\"\]\\evil""#),
            "{line}"
        );
        let block = &line[line.find('[').expect("a block")..];
        assert!(
            block.starts_with("[vigil@32473 ") && block.contains("] \u{feff}"),
            "the block still ends where it should: {block}"
        );
    }
}
