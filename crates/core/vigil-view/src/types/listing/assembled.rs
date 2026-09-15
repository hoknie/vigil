use crate::types::RowKey;

#[derive(Debug, Clone, PartialEq)]
pub enum Assembled {
    Ordered(Vec<usize>),
    Built(Vec<RowKey>),
}
