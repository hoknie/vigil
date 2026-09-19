pub fn running(_executable: &str, _uid: u32) -> Result<Vec<u32>, String> {
    Err(format!(
        "the processes running a program are found on Linux and on macOS, and this is {}",
        std::env::consts::OS
    ))
}

pub fn still_running(_pid: u32, _executable: &str, _uid: Option<u32>) -> bool {
    false
}
