pub(super) fn unquote(raw: &str) -> &str {
    raw.trim_matches(['"', '\''])
}

pub(super) fn text(raw: &str) -> (Option<String>, bool) {
    if raw.starts_with('"') || raw.starts_with('\'') {
        return (Some(unquote(raw).to_string()), false);
    }
    if raw == "(null)" || raw == "?" || raw.is_empty() {
        return (None, false);
    }
    if let Some(bytes) = from_hex(raw) {
        return match String::from_utf8(bytes) {
            Ok(text) => (Some(text), false),
            Err(error) => (
                Some(String::from_utf8_lossy(error.as_bytes()).into_owned()),
                true,
            ),
        };
    }
    (Some(raw.to_string()), false)
}

pub(super) fn from_hex(raw: &str) -> Option<Vec<u8>> {
    if raw.is_empty() || !raw.len().is_multiple_of(2) {
        return None;
    }
    let mut out = Vec::with_capacity(raw.len() / 2);
    let digits = raw.as_bytes();
    for pair in digits.chunks(2) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        out.push((high * 16 + low) as u8);
    }
    Some(out)
}
