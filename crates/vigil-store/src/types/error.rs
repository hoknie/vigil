use std::fmt;
#[derive(Debug)]
pub enum StoreError {
    Io(String),
    Corrupt(String),
    Full(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Io(what) => write!(f, "store io: {what}"),
            StoreError::Corrupt(what) => write!(f, "store is corrupt: {what}"),
            StoreError::Full(what) => write!(f, "store is at its cap: {what}"),
        }
    }
}

impl std::error::Error for StoreError {}
