#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Launched {
    pub seconds: u64,
    pub milliseconds: u32,
    pub sequence: u64,
    pub pid: u32,
    pub ppid: u32,
    pub auid: u32,
    pub uid: u32,
    pub euid: u32,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: Option<String>,
}
