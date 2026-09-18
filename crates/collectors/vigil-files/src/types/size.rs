use std::fmt;

use serde::Deserialize;
use serde::de::{self, Deserializer, Visitor};

const UNITS: &[(&str, u64)] = &[
    ("gb", 1024 * 1024 * 1024),
    ("mb", 1024 * 1024),
    ("kb", 1024),
    ("b", 1),
    ("", 1),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Size(pub u64);

impl Size {
    pub fn bytes(self) -> u64 {
        self.0
    }

    pub fn read(written: &str) -> Result<Size, String> {
        let text = written.trim().to_ascii_lowercase();
        let digits_end = text
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(text.len());
        let (digits, unit) = text.split_at(digits_end);
        let refused = || {
            format!(
                "{written:?} is not a size: write bytes as a number, or 512kb, 30mb, 1gb \
                 (each 1024 of the one before)"
            )
        };
        let count: u64 = digits.parse().map_err(|_| refused())?;
        let by = UNITS
            .iter()
            .find(|(name, _)| *name == unit.trim())
            .map(|(_, by)| *by)
            .ok_or_else(refused)?;

        count.checked_mul(by).map(Size).ok_or_else(refused)
    }

    pub fn shown(bytes: u64) -> String {
        for (name, by) in UNITS {
            if *by > 1 && bytes >= *by && bytes.is_multiple_of(*by) {
                return format!("{}{name}", bytes / by);
            }
        }
        bytes.to_string()
    }
}

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Size, D::Error> {
        deserializer.deserialize_any(Written)
    }
}

struct Written;

impl Visitor<'_> for Written {
    type Value = Size;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a number of bytes, or a size such as 512kb, 30mb or 1gb")
    }

    fn visit_u64<E: de::Error>(self, bytes: u64) -> Result<Size, E> {
        Ok(Size(bytes))
    }

    fn visit_i64<E: de::Error>(self, bytes: i64) -> Result<Size, E> {
        u64::try_from(bytes)
            .map(Size)
            .map_err(|_| E::custom(format!("{bytes} is not a size: a size is never below zero")))
    }

    fn visit_str<E: de::Error>(self, written: &str) -> Result<Size, E> {
        Size::read(written).map_err(E::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_size_is_read_as_bytes_or_with_a_unit_each_a_thousand_and_twenty_four_of_the_one_before() {
        for (written, bytes) in [
            ("512kb", 512 * 1024),
            ("30mb", 30 * 1024 * 1024),
            ("1gb", 1024 * 1024 * 1024),
            ("4096", 4096),
            ("4096b", 4096),
            (" 8 MB ", 8 * 1024 * 1024),
        ] {
            assert_eq!(Size::read(written), Ok(Size(bytes)), "{written}");
        }
    }

    #[test]
    fn a_size_this_agent_cannot_read_is_refused_in_words_that_name_the_forms_it_can() {
        for written in [
            "",
            "mb",
            "8 megabytes",
            "1.5mb",
            "-1",
            "99999999999999999999gb",
        ] {
            let refusal = Size::read(written).expect_err("must not be accepted");
            assert!(refusal.contains("30mb"), "{written}: {refusal}");
        }
    }

    #[test]
    fn a_size_in_the_file_may_be_a_bare_number_or_a_word() {
        let bare: Size = serde_json::from_str("8388608").expect("reads");
        let word: Size = serde_json::from_str("\"8mb\"").expect("reads");

        assert_eq!(bare, word);
        assert!(serde_json::from_str::<Size>("-5").is_err());
        assert!(serde_json::from_str::<Size>("true").is_err());
    }

    #[test]
    fn a_size_is_written_back_in_the_largest_unit_that_holds_it_whole() {
        assert_eq!(Size::shown(8 * 1024 * 1024), "8mb");
        assert_eq!(Size::shown(1024 * 1024 * 1024), "1gb");
        assert_eq!(Size::shown(1536), "1536");
        assert_eq!(Size::shown(2048), "2kb");
        assert_eq!(Size::shown(0), "0");
        for bytes in [1, 1023, 4096, 3 * 1024 * 1024, 1_048_577] {
            assert_eq!(Size::read(&Size::shown(bytes)), Ok(Size(bytes)), "{bytes}");
        }
    }
}
