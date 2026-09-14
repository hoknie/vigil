#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ours {
    pub uid: u32,
    pub pid: u32,
}

impl Ours {
    pub fn here() -> Ours {
        Ours {
            uid: rustix::process::getuid().as_raw(),
            pid: std::process::id(),
        }
    }
}
