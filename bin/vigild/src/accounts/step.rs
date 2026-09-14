use super::utility::Utility;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Run {
        utility: Utility,
        arguments: Vec<String>,
    },
    Directory {
        path: String,
        uid: u32,
        gid: u32,
        mode: u32,
    },
    Write {
        path: String,
        text: String,
        uid: u32,
        gid: u32,
        mode: u32,
    },
    Remove {
        path: String,
        holder: Option<u32>,
    },
    Sudoers {
        path: String,
        text: String,
    },
    Signal {
        pid: u32,
        uid: Option<u32>,
    },
}
