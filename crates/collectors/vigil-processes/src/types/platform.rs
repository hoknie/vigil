pub struct Platform {
    pub unresolved_reason: &'static str,
    pub writable_paths: &'static [&'static str],
}

pub const LINUX: Platform = Platform {
    unresolved_reason: "some executables could not be read: those programs are not in this reading (needs CAP_SYS_PTRACE, or root without a restricted capability set)",
    writable_paths: &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"],
};

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub const MACOS: Platform = Platform {
    unresolved_reason: "some executables could not be read: those programs are not in this reading (macOS names the program of a process whose file was deleted only to its own account and to root; run as root)",
    writable_paths: &[
        "/tmp/",
        "/var/tmp/",
        "/private/tmp/",
        "/private/var/tmp/",
        "/private/var/folders/",
        "/Users/",
    ],
};

impl Platform {
    pub fn writable(&self, executable: &str) -> bool {
        self.writable_paths
            .iter()
            .any(|writable| executable.starts_with(writable))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_program_in_a_directory_anybody_may_write_to_on_macos_is_one_under_its_real_name() {
        for path in [
            "/private/tmp/.x/nc",
            "/private/var/folders/zz/abc/T/payload",
            "/Users/Shared/agent",
            "/tmp/nc",
        ] {
            assert!(
                MACOS.writable(path),
                "{path}: macOS names the program of a process by the path its file really \
                 lives at, and /tmp is a link to /private/tmp"
            );
        }
        for path in [
            "/usr/bin/nc",
            "/Applications/Safari.app/Contents/MacOS/Safari",
        ] {
            assert!(!MACOS.writable(path), "{path}");
        }
    }

    #[test]
    fn the_directories_a_linux_reading_marks_are_the_ones_it_marked_before_macos_was_added() {
        assert_eq!(
            LINUX.writable_paths,
            &["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"]
        );
        assert!(!LINUX.writable("/private/tmp/nc"));
    }
}
