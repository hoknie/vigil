use std::fmt;

#[derive(Debug)]
pub enum CollectError {
    Denied(String),
    Absent(String),
    Unreadable(String),
    Budget(String),
}

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectError::Denied(what) => write!(f, "not permitted to read {what}"),
            CollectError::Absent(what) => write!(f, "{what} is not present on this system"),
            CollectError::Unreadable(what) => write!(f, "could not parse {what}"),
            CollectError::Budget(what) => write!(f, "reading {what} exceeded its budget"),
        }
    }
}

impl std::error::Error for CollectError {}
