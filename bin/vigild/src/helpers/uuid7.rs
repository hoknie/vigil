use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

use vigil_model::Uuid7;

pub fn mint() -> Uuid7 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as u64)
        .unwrap_or(0);
    format(millis, random_bytes())
}

fn random_bytes() -> [u8; 10] {
    let mut bytes = [0u8; 10];
    match File::open("/dev/urandom").and_then(|mut pool| pool.read_exact(&mut bytes)) {
        Ok(()) => {}
        Err(error) => {
            panic!("/dev/urandom is unreadable ({error}): no identifier can be minted");
        }
    }
    bytes
}

fn format(millis: u64, random: [u8; 10]) -> Uuid7 {
    let mut bytes = [0u8; 16];
    bytes[..6].copy_from_slice(&millis.to_be_bytes()[2..]);
    bytes[6..].copy_from_slice(&random);

    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_the_version_and_variant_a_receiver_checks() {
        let id = format(1_788_869_042_104, [0xff; 10]);

        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "7", "version nibble: {id}");
        assert!(
            matches!(&id[19..20], "8" | "9" | "a" | "b"),
            "variant bits: {id}"
        );
    }

    #[test]
    fn ids_minted_later_sort_after_ids_minted_earlier() {
        let earlier = format(1_788_869_042_104, [0x00; 10]);
        let later = format(1_788_869_042_105, [0x00; 10]);

        assert!(earlier < later, "{earlier} !< {later}");
    }

    #[test]
    fn two_ids_from_the_same_millisecond_still_differ() {
        assert_ne!(mint(), mint());
    }
}
