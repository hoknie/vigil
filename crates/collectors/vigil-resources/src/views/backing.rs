use serde_json::Value;

use super::fields::text;
use crate::types::{Backing, Family};

pub const HEADING: &str = "storage|";

pub fn storage(item: &Value) -> &str {
    text(item, "storage")
}

pub fn backing(item: &Value) -> Backing {
    Backing::of(text(item, "storage_from")).unwrap_or(Backing::Unnamed)
}

pub fn heading_of(item: &Value) -> String {
    format!("{HEADING}{}|{}", backing(item).name(), storage(item))
}

pub fn taken_apart(heading: &str) -> Option<(Backing, &str)> {
    let (from, name) = heading.strip_prefix(HEADING)?.split_once('|')?;

    Some((Backing::of(from)?, name))
}

pub fn is_a_filesystem(key: &str) -> bool {
    Family::of(key) == Some(Family::Filesystem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_heading_carries_both_the_name_of_the_storage_and_how_this_host_named_it() {
        let filesystem = fixture::filesystem("/var", Some(35), Some(85));

        let heading = heading_of(&filesystem);

        assert_eq!(heading, "storage|disk|sda");
        assert_eq!(taken_apart(&heading), Some((Backing::Disk, "sda")));
    }

    #[test]
    fn a_reading_from_a_build_that_did_not_name_the_storage_is_gathered_and_not_dropped() {
        let mut older = fixture::filesystem("/var", Some(35), Some(85));
        older["storage"] = Value::Null;
        older["storage_from"] = Value::Null;

        assert_eq!(backing(&older), Backing::Unnamed);
        assert_eq!(
            heading_of(&older),
            "storage|unnamed|?",
            "a baseline written before this build knew about block devices still has to land \
             somewhere on the screen"
        );
    }

    #[test]
    fn a_heading_of_a_shape_this_build_does_not_know_is_taken_apart_into_nothing() {
        assert_eq!(taken_apart("storage|sda"), None);
        assert_eq!(taken_apart("fs|/var"), None);
        assert_eq!(taken_apart("storage|from-a-later-version|sda"), None);
    }
}
