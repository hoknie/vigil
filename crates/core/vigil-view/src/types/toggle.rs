#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Toggle {
    pub key: char,
    pub name: &'static str,
}

impl Toggle {
    pub fn new(key: char, name: &'static str) -> Toggle {
        Toggle { key, name }
    }
}
