#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    pub kept: u64,
    pub skipped: u64,
    pub oversized: u64,
    pub compactions: u64,
    pub dropped: u64,
}
