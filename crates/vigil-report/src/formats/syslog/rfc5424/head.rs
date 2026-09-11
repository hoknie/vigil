use vigil_model::{Finding, Host};

use super::Rfc5424;
use super::text::{ascii_field, timestamp};
use crate::formats::syslog::urgency;

const HOSTNAME_CAP: usize = 128;
const APP_NAME_CAP: usize = 48;
const PROC_ID_CAP: usize = 12;
const MSG_ID_CAP: usize = 32;

impl Rfc5424 {
    pub(super) fn head(&self, finding: &Finding, host: &Host, sent_at: &str) -> String {
        let priority = urgency::priority(self.facility, &finding.severity);
        let timestamp = timestamp(&finding.observed_at)
            .or_else(|| timestamp(sent_at))
            .unwrap_or_else(|| "-".to_string());
        let hostname = ascii_field(host.fqdn.as_deref().unwrap_or(&host.hostname), HOSTNAME_CAP);
        let app_name = ascii_field(&self.app_name, APP_NAME_CAP);
        let process_id = ascii_field(&self.process_id.to_string(), PROC_ID_CAP);
        let message_id = ascii_field(finding.kind.as_str(), MSG_ID_CAP);

        format!("<{priority}>1 {timestamp} {hostname} {app_name} {process_id} {message_id} ")
    }
}

#[cfg(test)]
mod tests {
    use super::super::fixture::{finding, format, host};
    use super::*;
    use crate::formats::syslog::SyslogFacility;
    use vigil_model::{Kind, KnownKind, Severity};

    #[test]
    fn the_time_on_the_line_is_when_the_host_did_it_not_when_we_sent_it() {
        let line = format().line(&finding(), &host(), "2026-09-09T18:00:00.000Z");

        assert!(line.contains("2026-09-09T12:04:02.311Z"), "{line}");
        assert!(!line.contains("18:00:00"), "{line}");
    }

    #[test]
    fn a_timestamp_that_is_not_one_falls_back_instead_of_breaking_the_header() {
        let mut broken = finding();
        broken.observed_at = "yesterday afternoon".into();

        let line = format().line(&broken, &host(), "2026-09-09T18:00:00.000Z");

        assert_eq!(
            line.split(' ').nth(1).expect("a header"),
            "2026-09-09T18:00:00.000Z",
            "the timestamp is still the second field: {line}"
        );
    }

    #[test]
    fn the_severity_of_the_finding_is_the_severity_of_the_line() {
        let facility = SyslogFacility::parse("daemon").expect("known");
        for (severity, expected) in [
            (Severity::Critical, 3 * 8 + 2),
            (Severity::High, 3 * 8 + 3),
            (Severity::Medium, 3 * 8 + 4),
            (Severity::Low, 3 * 8 + 5),
            (Severity::Info, 3 * 8 + 6),
            (Severity::Unknown("worse".into()), 3 * 8 + 4),
        ] {
            let mut one = finding();
            one.severity = severity.clone();
            let line = Rfc5424::new(facility, "vigil", 1).line(&one, &host(), "");

            assert!(
                line.starts_with(&format!("<{expected}>1 ")),
                "{severity} became {line}"
            );
        }
    }

    #[test]
    fn a_kind_longer_than_the_message_id_field_is_cut_there_and_whole_in_the_data() {
        let mut long = finding();
        long.kind = Kind::Known(KnownKind::UserGroupPrivilegedMemberAdded);

        let line = format().line(&long, &host(), "");

        let message_id = line.split(' ').nth(5).expect("a header");
        assert_eq!(message_id.len(), MSG_ID_CAP, "{line}");
        assert!(
            line.contains(r#"kind="user.group.privileged_member_added""#),
            "the whole kind still travels: {line}"
        );
    }

    #[test]
    fn a_host_with_no_name_gets_the_nil_value_rather_than_an_empty_field() {
        let mut anonymous = host();
        anonymous.fqdn = None;
        anonymous.hostname = String::new();

        let line = format().line(&finding(), &anonymous, "");

        assert!(line.contains(" - vigil 4242 "), "{line}");
    }
}
