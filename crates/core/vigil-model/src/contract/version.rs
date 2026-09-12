use std::fmt;

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: SchemaVersion = SchemaVersion { major: 1, minor: 2 };

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SchemaVersion {
    pub major: u16,
    pub minor: u16,
}

impl SchemaVersion {
    pub fn accepts(self, other: SchemaVersion) -> bool {
        other.major == self.major || other.major + 1 == self.major
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl From<SchemaVersion> for String {
    fn from(v: SchemaVersion) -> Self {
        v.to_string()
    }
}

impl TryFrom<String> for SchemaVersion {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let (major, minor) = s
            .split_once('.')
            .ok_or_else(|| format!("schema_version {s:?} is not MAJOR.MINOR"))?;
        Ok(SchemaVersion {
            major: major
                .parse()
                .map_err(|_| format!("schema_version {s:?} has a non-numeric major"))?,
            minor: minor
                .parse()
                .map_err(|_| format!("schema_version {s:?} has a non-numeric minor"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(major: u16, minor: u16) -> SchemaVersion {
        SchemaVersion { major, minor }
    }

    #[test]
    fn accepts_own_major_at_any_minor_and_the_previous_major() {
        let me = v(2, 3);
        assert!(me.accepts(v(2, 0)));
        assert!(me.accepts(v(2, 9)), "a higher minor only adds fields");
        assert!(me.accepts(v(1, 7)), "N-1 stays supported");
        assert!(
            !me.accepts(v(3, 0)),
            "a newer major may have removed something"
        );
        assert!(!me.accepts(v(0, 1)));
    }

    #[test]
    fn round_trips_through_its_wire_form() {
        let text = String::from(SCHEMA_VERSION);
        assert_eq!(
            SchemaVersion::try_from(text.clone()).expect("parses"),
            SCHEMA_VERSION
        );
        assert_eq!(
            text,
            format!("{}.{}", SCHEMA_VERSION.major, SCHEMA_VERSION.minor)
        );

        assert_eq!(
            SchemaVersion::try_from("1.0".to_string()).expect("parses"),
            v(1, 0)
        );
        assert!(SchemaVersion::try_from("1".to_string()).is_err());
    }
}
