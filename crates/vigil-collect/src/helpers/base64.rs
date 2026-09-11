const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode_unpadded(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        let characters = chunk.len() + 1;
        for index in 0..characters {
            let sextet = (triple >> (18 - index * 6)) & 0x3f;
            out.push(ALPHABET[sextet as usize] as char);
        }
    }

    out
}

pub fn decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut accumulator = 0u32;
    let mut bits = 0u32;

    for character in text.bytes() {
        if character == b'=' {
            break;
        }
        let value = match character {
            b'A'..=b'Z' => character - b'A',
            b'a'..=b'z' => character - b'a' + 26,
            b'0'..=b'9' => character - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32;

        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((accumulator >> bits) as u8);
        }
    }

    if bits > 0 && (accumulator & ((1 << bits) - 1)) != 0 {
        return None;
    }

    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_standards_own_test_vectors_without_the_padding() {
        assert_eq!(encode_unpadded(b""), "");
        assert_eq!(encode_unpadded(b"f"), "Zg");
        assert_eq!(encode_unpadded(b"fo"), "Zm8");
        assert_eq!(encode_unpadded(b"foo"), "Zm9v");
        assert_eq!(encode_unpadded(b"foob"), "Zm9vYg");
        assert_eq!(encode_unpadded(b"fooba"), "Zm9vYmE");
        assert_eq!(encode_unpadded(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn decodes_what_it_encodes_and_the_padded_form_too() {
        assert_eq!(decode("Zm9vYmFy").as_deref(), Some(&b"foobar"[..]));
        assert_eq!(decode("Zm9vYg==").as_deref(), Some(&b"foob"[..]));
        assert_eq!(decode("Zm9vYmE=").as_deref(), Some(&b"fooba"[..]));
    }

    #[test]
    fn refuses_text_that_is_not_base64_instead_of_returning_a_prefix() {
        assert_eq!(decode("alice@laptop"), None, "a comment is not a key");
        assert_eq!(decode("no-touch-host"), None);
        assert!(decode("Zm9v!!").is_none());
    }
}
