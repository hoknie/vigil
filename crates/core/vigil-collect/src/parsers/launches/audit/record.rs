use super::fields::{fields, value};
use super::text::unquote;

pub(super) struct Record {
    pub(super) kind: String,
    pub(super) event: String,
    pub(super) fields: Vec<(String, String)>,
}

impl Record {
    pub(super) fn parse(line: &str) -> Option<Record> {
        let fields = fields(line);
        let kind = unquote(value(&fields, "type")?).to_string();
        let message = value(&fields, "msg")?;
        let event = message
            .strip_prefix("audit(")?
            .trim_end_matches(':')
            .trim_end_matches(')')
            .to_string();

        Some(Record {
            kind,
            event,
            fields,
        })
    }
}
