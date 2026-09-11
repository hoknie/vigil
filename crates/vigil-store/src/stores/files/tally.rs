use crate::Dropped;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Tally {
    pub damaged_on_open: usize,
    pub journal_absent_on_open: bool,
    pub compactions: u64,
    pub dropped: Dropped,
}
