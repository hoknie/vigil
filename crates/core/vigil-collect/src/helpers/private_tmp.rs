const PRIVATE_DIRECTORIES: &[&str] = &["/tmp", "/var/tmp"];

pub fn shown_to_the_agent(path: &str) -> bool {
    !PRIVATE_DIRECTORIES.iter().any(|root| {
        path.strip_prefix(root)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_two_directories_the_shipped_unit_keeps_to_itself_are_not_the_host_directories() {
        assert!(!shown_to_the_agent("/tmp"));
        assert!(!shown_to_the_agent("/tmp/payload"));
        assert!(!shown_to_the_agent("/var/tmp"));
        assert!(!shown_to_the_agent("/var/tmp/build/tool"));
    }

    #[test]
    fn a_directory_that_merely_starts_with_the_same_letters_is_the_host_directory() {
        assert!(shown_to_the_agent("/tmpfiles"));
        assert!(shown_to_the_agent("/var/tmpfs/thing"));
        assert!(shown_to_the_agent("/home/deploy/tmp/tool"));
        assert!(shown_to_the_agent("/usr/bin/nc"));
        assert!(shown_to_the_agent(""));
    }
}
