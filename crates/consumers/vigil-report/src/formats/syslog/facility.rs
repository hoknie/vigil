#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyslogFacility(u8);

const FACILITIES: &[(&str, u8)] = &[
    ("kern", 0),
    ("user", 1),
    ("mail", 2),
    ("daemon", 3),
    ("auth", 4),
    ("syslog", 5),
    ("lpr", 6),
    ("news", 7),
    ("uucp", 8),
    ("cron", 9),
    ("authpriv", 10),
    ("ftp", 11),
    ("ntp", 12),
    ("audit", 13),
    ("alert", 14),
    ("clock", 15),
    ("local0", 16),
    ("local1", 17),
    ("local2", 18),
    ("local3", 19),
    ("local4", 20),
    ("local5", 21),
    ("local6", 22),
    ("local7", 23),
];

impl SyslogFacility {
    pub fn parse(name: &str) -> Result<Self, String> {
        let wanted = name.trim().to_ascii_lowercase();
        match FACILITIES.iter().find(|(known, _)| *known == wanted) {
            Some((_, code)) => Ok(SyslogFacility(*code)),
            None => Err(format!(
                "unknown syslog facility '{name}'. Known facilities: {}",
                Self::names().join(", ")
            )),
        }
    }

    pub fn code(self) -> u8 {
        self.0
    }

    pub fn name(self) -> &'static str {
        FACILITIES
            .iter()
            .find(|(_, code)| *code == self.0)
            .map(|(name, _)| *name)
            .unwrap_or("unknown")
    }

    pub fn names() -> Vec<&'static str> {
        FACILITIES.iter().map(|(name, _)| *name).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_accepted_name_maps_to_its_own_code_and_back() {
        for (name, code) in FACILITIES {
            let facility = SyslogFacility::parse(name).expect("a listed name is accepted");
            assert_eq!(facility.code(), *code, "{name}");
            assert_eq!(facility.name(), *name, "{name}");
        }
    }

    #[test]
    fn a_misspelled_facility_is_refused_and_the_message_says_what_was_allowed() {
        let error = SyslogFacility::parse("lokal0").expect_err("a typo must not be accepted");

        assert!(error.contains("lokal0"), "{error}");
        assert!(
            error.contains("local0") && error.contains("authpriv"),
            "the alternatives belong in the message, not in the documentation: {error}"
        );
    }

    #[test]
    fn the_name_is_read_the_way_an_operator_writes_it() {
        assert_eq!(
            SyslogFacility::parse("  LOCAL4 ")
                .expect("case and blanks")
                .code(),
            20
        );
    }

    #[test]
    fn an_alias_two_implementations_disagree_about_is_refused_rather_than_guessed() {
        SyslogFacility::parse("security").expect_err("4 or 13 is not something to guess at");
    }
}
