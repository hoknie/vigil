#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Health {
    Ok,
    Degraded(String),
    Unavailable(String),
}
