use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub uid: u32,
    pub home: Option<PathBuf>,
}
