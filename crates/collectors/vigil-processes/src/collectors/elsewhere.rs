pub fn running(_executable: &str, _uid: u32) -> Result<Vec<u32>, String> {
    Err("the processes running a program are found in a Linux /proc".to_string())
}

pub fn still_running(_pid: u32, _executable: &str, _uid: Option<u32>) -> bool {
    false
}
