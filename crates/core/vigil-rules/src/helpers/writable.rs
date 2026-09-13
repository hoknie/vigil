pub fn is_writable_path(executable: &str) -> bool {
    ["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"]
        .iter()
        .any(|writable| executable.starts_with(writable))
}
