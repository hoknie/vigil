#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Switched {
    Changed(String),
    AlreadySo,
    NotOurs(String),
}
