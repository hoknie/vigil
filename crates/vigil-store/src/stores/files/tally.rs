use crate::Dropped;

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Tally {
    pub damaged_on_open: usize,
    pub compactions: u64,
    pub dropped: Dropped,
}
