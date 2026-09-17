use crate::helpers::quoting::quoted;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watch {
    pub path: String,
    pub ceiling_bytes: Option<u64>,
}

impl Watch {
    pub fn of(path: impl Into<String>, ceiling_bytes: Option<u64>) -> Watch {
        Watch {
            path: path.into(),
            ceiling_bytes,
        }
    }

    pub(crate) fn lines(&self, indent: &str) -> Vec<String> {
        match self.ceiling_bytes {
            None => vec![format!("{indent}- {}", quoted(&self.path))],
            Some(ceiling_bytes) => vec![
                format!("{indent}- path: {}", quoted(&self.path)),
                format!("{indent}  ceiling_bytes: {ceiling_bytes}"),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_with_nothing_said_about_it_goes_into_the_file_as_one_quoted_word() {
        assert_eq!(
            Watch::of("/etc/hosts", None).lines("    "),
            vec!["    - \"/etc/hosts\""]
        );
    }

    #[test]
    fn a_path_hashed_to_its_own_ceiling_is_written_as_a_pair_the_agent_reads_back() {
        assert_eq!(
            Watch::of("/etc/ssl/certs/ca.crt", Some(8_388_608)).lines("    "),
            vec![
                "    - path: \"/etc/ssl/certs/ca.crt\"",
                "      ceiling_bytes: 8388608",
            ],
            "the second line sits under the first key and not under the dash, or what comes \
             back is a list of two things rather than one path with a ceiling"
        );
    }

    #[test]
    fn a_path_holding_a_quotation_mark_goes_in_quoted_and_comes_back_whole() {
        let lines = Watch::of("/etc/a\"b", None).lines("  ");

        assert_eq!(lines, vec!["  - \"/etc/a\\\"b\""]);
    }
}
