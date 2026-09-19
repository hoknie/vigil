#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessEntry {
    pub pid: u32,
    pub parent: u32,
    pub real_uid: u32,
    pub effective_uid: u32,
    pub name: String,
    pub zombie: bool,
}
