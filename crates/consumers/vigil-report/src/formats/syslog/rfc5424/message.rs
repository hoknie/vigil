use serde_json::Value;
use vigil_model::Finding;

use super::Rfc5424;
use super::fields::Fields;
use super::text::{clamp, compact, printable, scalar};

pub(super) const BOM: &str = "\u{feff}";

const TITLE_CAP: usize = 256;

impl Rfc5424 {
    pub(super) fn message_fields(&self, finding: &Finding) -> Fields {
        let mut fields = Fields::default();

        let (title, cut) = clamp(&printable(&finding.title), TITLE_CAP);
        fields.clamped += cut;
        fields.items.push(match title.is_empty() {
            true => finding.kind.as_str().to_string(),
            false => format!("{title} |"),
        });

        fields.pair("object", &finding.subject.object);
        match &finding.subject.key {
            Value::Object(map) => {
                for (name, value) in map {
                    if let Some(text) = scalar(value) {
                        fields.pair(name, &text);
                    }
                }
            }
            other => {
                if let Some(text) = scalar(other) {
                    fields.pair("subject", &text);
                }
            }
        }

        if finding.occurrences > 1 {
            fields.pair("occurrences", &finding.occurrences.to_string());
        }

        for item in &finding.evidence {
            fields.pair(&item.kind, &item.value);
        }

        if !finding.redacted.is_empty() {
            fields.pair("redacted", &finding.redacted.join(" "));
        }

        if let Some(before) = &finding.before {
            fields.pair("before", &compact(before));
        }
        if let Some(after) = &finding.after {
            fields.pair("after", &compact(after));
        }

        fields
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{finding, format, host};

    #[test]
    fn a_repeat_says_how_many_times_it_has_happened() {
        let mut again = finding();
        again.occurrences = 12;

        let line = format().line(&again, &host(), "");
        assert!(line.contains(r#"occurrences="12""#), "{line}");
        assert_eq!(
            line.matches(r#"occurrences="12""#).count(),
            2,
            "once for the parser, once for the person: {line}"
        );
    }
}
