use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Done {
    pub said: Vec<String>,
    pub entries: usize,
    pub file: Option<PathBuf>,
}
