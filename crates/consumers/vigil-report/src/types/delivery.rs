#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    Accepted {
        accepted: u64,
        duplicates: u64,
    },
    Partial {
        accepted: u64,
        rejected: Vec<String>,
    },
}
