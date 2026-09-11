#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeLine {
    pub mark: Mark,
    pub path: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Added,
    Removed,
    Same,
}

impl Mark {
    pub fn symbol(self) -> char {
        match self {
            Mark::Added => '+',
            Mark::Removed => '-',
            Mark::Same => ' ',
        }
    }
}
