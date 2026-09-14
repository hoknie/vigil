use crate::parsers::{AuthorizedKey, parse_authorized_keys};

use super::one_line::one_line;

pub const LINE_HINT: &str =
    "one line as it stands in authorized_keys: [options] algorithm base64 [comment]";

pub fn key_line(value: &str) -> Result<AuthorizedKey, String> {
    if value.trim().is_empty() {
        return Err(
            "the key line is empty: paste the line as it would stand in authorized_keys"
                .to_string(),
        );
    }
    one_line("a key line", value)?;

    let mut keys = parse_authorized_keys(value);
    match keys.len() {
        1 => Ok(keys.remove(0)),
        0 => Err(
            "that line holds no key this agent can read: expected [options] algorithm base64 \
             [comment], as in ssh-ed25519 AAAA… person@laptop"
                .to_string(),
        ),
        _ => Err("that is more than one key: add them one at a time".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

    #[test]
    fn a_line_the_reader_of_authorized_keys_reads_as_one_key_is_accepted() {
        let key = key_line(KEY).expect("one key");
        assert_eq!(key.algorithm, "ssh-ed25519");
        assert_eq!(key.comment.as_deref(), Some("person@laptop"));
    }

    #[test]
    fn a_line_that_is_not_a_key_is_refused_rather_than_written_into_a_file_sshd_reads() {
        for line in ["", "hello", "ssh-ed25519 notbase64"] {
            assert!(key_line(line).is_err(), "{line:?}");
        }
    }

    #[test]
    fn two_lines_pasted_at_once_are_refused_because_a_key_line_is_one_line() {
        let said = key_line(&format!("{KEY}\n{KEY}")).expect_err("two lines");
        assert!(said.contains("line break"), "{said}");
    }
}
