use super::Placed;
use crate::types::RowKey;

#[derive(Debug, Clone, PartialEq)]
pub enum Assembled {
    Ordered(Vec<usize>),
    Gathered(Vec<Placed>),
    Built(Vec<RowKey>),
}
