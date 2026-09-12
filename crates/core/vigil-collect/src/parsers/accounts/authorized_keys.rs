use crate::helpers::{decode, encode_unpadded, sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedKey {
    pub algorithm: String,
    pub fingerprint: String,
    pub comment: Option<String>,
    pub options: Option<String>,
}

pub fn parse_authorized_keys(text: &str) -> Vec<AuthorizedKey> {
    let mut keys = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(key) = parse_line(line) {
            keys.push(key);
        }
    }

    keys
}

fn parse_line(line: &str) -> Option<AuthorizedKey> {
    let fields = split_outside_quotes(line);

    for index in 0..fields.len().saturating_sub(1) {
        if !looks_like_algorithm(&fields[index]) {
            continue;
        }
        let Some(blob) = decode(&fields[index + 1]) else {
            continue;
        };
        if !blob_names(&blob, &fields[index]) {
            continue;
        }

        let options = match index {
            0 => None,
            _ => Some(fields[..index].join(" ")),
        };
        let comment = match fields.len() > index + 2 {
            true => Some(fields[index + 2..].join(" ")),
            false => None,
        };

        return Some(AuthorizedKey {
            algorithm: fields[index].clone(),
            fingerprint: fingerprint(&blob),
            comment,
            options,
        });
    }

    None
}

pub fn fingerprint(blob: &[u8]) -> String {
    format!("SHA256:{}", encode_unpadded(&sha256(blob)))
}

fn blob_names(blob: &[u8], algorithm: &str) -> bool {
    if blob.len() < 4 {
        return false;
    }
    let length = u32::from_be_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    if length == 0 || blob.len() < 4 + length {
        return false;
    }
    blob[4..4 + length] == *algorithm.as_bytes()
}

fn looks_like_algorithm(field: &str) -> bool {
    field.starts_with("ssh-")
        || field.starts_with("ecdsa-")
        || field.starts_with("sk-")
        || field.starts_with("webauthn-")
}

fn split_outside_quotes(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;

    for character in line.chars() {
        match character {
            _ if escaped => {
                current.push(character);
                escaped = false;
            }
            '\\' if quoted => {
                current.push(character);
                escaped = true;
            }
            '"' => {
                quoted = !quoted;
                current.push(character);
            }
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    fields.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        fields.push(current);
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    const ED25519: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr deploy@builder";
    const ED25519_FINGERPRINT: &str = "SHA256:ie96zLdp9BkGxjqBG3AldJ2imVMNe8sXmmyorODpGCM";

    #[test]
    fn reads_the_algorithm_the_fingerprint_and_the_comment_but_not_the_key() {
        let keys = parse_authorized_keys(ED25519);

        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].algorithm, "ssh-ed25519");
        assert_eq!(keys[0].fingerprint, ED25519_FINGERPRINT);
        assert_eq!(keys[0].comment.as_deref(), Some("deploy@builder"));
        assert_eq!(keys[0].options, None);

        let printed = format!("{:?}", keys[0]);
        assert!(
            !printed.contains("AAAAC3Nza"),
            "the key body must not survive the parse: {printed}"
        );
    }

    #[test]
    fn an_option_list_with_a_quoted_command_in_it_does_not_swallow_the_key() {
        let line = format!(r#"command="/usr/bin/backup --all",no-pty,from="10.0.0.0/8" {ED25519}"#);

        let keys = parse_authorized_keys(&line);

        assert_eq!(keys.len(), 1, "{keys:?}");
        assert_eq!(keys[0].fingerprint, ED25519_FINGERPRINT);
        assert_eq!(
            keys[0].options.as_deref(),
            Some(r#"command="/usr/bin/backup --all",no-pty,from="10.0.0.0/8""#)
        );
        assert_eq!(keys[0].comment.as_deref(), Some("deploy@builder"));
    }

    #[test]
    fn a_comment_with_spaces_stays_one_comment() {
        let line = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr alice on the old laptop";

        let keys = parse_authorized_keys(line);

        assert_eq!(keys[0].comment.as_deref(), Some("alice on the old laptop"));
    }

    #[test]
    fn skips_what_is_not_a_key_instead_of_inventing_one() {
        let text = "\
# the deploy key, added 2026-03-01

not-a-key at all
ssh-ed25519 this-is-not-base64 broken
ssh-rsa AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr mislabelled
";

        assert!(parse_authorized_keys(text).is_empty());
    }

    #[test]
    fn two_keys_in_one_file_are_two_keys() {
        let second = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFmXA1pBTuHBFwlqBUwPMTFvOSlqDGBWZm2SsJ3TmiVJ alice@laptop";
        let keys = parse_authorized_keys(&format!("{ED25519}\n{second}\n"));

        assert_eq!(keys.len(), 2);
        assert_ne!(
            keys[0].fingerprint, keys[1].fingerprint,
            "two different keys must not share a snapshot key"
        );
    }
}
