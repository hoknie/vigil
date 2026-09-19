#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Execution {
    pub id: String,
    pub auid: Option<u32>,
    pub executable: Option<String>,
    pub executable_lossy: bool,
    pub arguments: Vec<String>,
    pub arguments_lossy: bool,
    pub arguments_redacted: bool,
}
