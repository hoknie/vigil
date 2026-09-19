use super::boot_id::parse_boot_id;

pub fn parse_boot_session(bytes: &[u8]) -> Option<String> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());

    parse_boot_id(std::str::from_utf8(&bytes[..end]).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_a_mac_booted_into_is_read_as_the_identifier_of_the_boot() {
        assert_eq!(
            parse_boot_session(b"17E0BCD5-CABB-436C-90A9-99726818CE57\0").as_deref(),
            Some("17E0BCD5-CABB-436C-90A9-99726818CE57")
        );
    }

    #[test]
    fn a_value_that_is_not_an_identifier_is_refused() {
        assert_eq!(parse_boot_session(b"\0"), None);
        assert_eq!(parse_boot_session(b"not a boot\0"), None);
        assert_eq!(parse_boot_session(&[0xff, 0xfe, 0]), None);
    }
}
