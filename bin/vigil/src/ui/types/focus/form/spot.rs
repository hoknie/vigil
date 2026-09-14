#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    Back,
    Field(usize),
    Save,
    Cancel,
}
