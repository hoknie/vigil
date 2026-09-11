pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_every_byte_to_two_digits() {
        assert_eq!(hex(&[0x0a, 0xbc]), "0abc");
        assert_eq!(hex(&[0x00, 0xff]), "00ff");
        assert_eq!(hex(&[]), "");
    }
}
