#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Program {
    At(String),
    Deleted(String),
    Unresolved,
    Gone,
}
