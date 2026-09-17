use crate::types::BlockEntry;

pub trait BlockTree {
    fn entry(&self, name: &str) -> BlockEntry;
}
