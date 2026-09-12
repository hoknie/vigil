use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    File,
    Directory,
}

const FILE: &str = "file";

const DIRECTORY: &str = "directory";

const SETUID: u32 = 0o4000;

const SETGID: u32 = 0o2000;

const WRITABLE_BY_ANYONE: u32 = 0o0002;

const STICKY: u32 = 0o1000;

pub struct FileView<'a> {
    key: &'a str,
    value: &'a Value,
}

impl<'a> FileView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        FileView { key, value }
    }

    pub fn family(&self) -> Option<Family> {
        match self.key.split('|').next()? {
            FILE => Some(Family::File),
            DIRECTORY => Some(Family::Directory),
            _ => None,
        }
    }

    pub fn is(&self, family: Family) -> bool {
        self.family() == Some(family)
    }

    pub fn path(&self) -> &'a str {
        self.value["path"].as_str().unwrap_or("?")
    }

    pub fn present(&self) -> bool {
        self.value["present"].as_bool().unwrap_or(false)
    }

    pub fn digest(&self) -> Option<&'a str> {
        self.value["sha256"].as_str()
    }

    pub fn size(&self) -> u64 {
        self.value["size"].as_u64().unwrap_or(0)
    }

    pub fn mode(&self) -> Option<&'a str> {
        self.value["mode"].as_str()
    }

    pub fn bits(&self) -> Option<u32> {
        u32::from_str_radix(self.mode()?, 8).ok()
    }

    pub fn owner(&self) -> (Option<u64>, Option<u64>) {
        (self.value["uid"].as_u64(), self.value["gid"].as_u64())
    }

    pub fn runs_as_its_owner(&self) -> bool {
        self.bits()
            .is_some_and(|bits| bits & (SETUID | SETGID) != 0)
    }

    pub fn writable_by_anyone(&self) -> bool {
        self.bits()
            .is_some_and(|bits| bits & WRITABLE_BY_ANYONE != 0 && bits & STICKY == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    #[test]
    fn an_item_of_another_collector_belongs_to_no_family_here() {
        let socket = fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root");

        assert_eq!(FileView::new("tcp|0.0.0.0:443", &socket).family(), None);
        assert_eq!(FileView::new("fs|/var", &socket).family(), None);
    }

    #[test]
    fn a_watched_file_and_a_directory_of_the_path_are_two_families() {
        let file = fixture::watched_file("/etc/hosts", "0644", "a9");
        let directory = fixture::watched_directory("/usr/bin", "0755");

        assert!(FileView::new("file|/etc/hosts", &file).is(Family::File));
        assert!(FileView::new("directory|/usr/bin", &directory).is(Family::Directory));
    }

    #[test]
    fn a_program_that_runs_as_its_owner_is_told_from_one_that_runs_as_whoever_started_it() {
        for mode in ["4755", "2755", "6755"] {
            let file = fixture::watched_file("/usr/bin/at", mode, "a9");
            assert!(
                FileView::new("file|/usr/bin/at", &file).runs_as_its_owner(),
                "{mode}"
            );
        }
        for mode in ["0755", "0644", "0700"] {
            let file = fixture::watched_file("/usr/bin/at", mode, "a9");
            assert!(
                !FileView::new("file|/usr/bin/at", &file).runs_as_its_owner(),
                "{mode}"
            );
        }
    }

    #[test]
    fn a_directory_anyone_may_write_a_program_into_is_not_one_with_the_sticky_bit_on_it() {
        let open = fixture::watched_directory("/usr/local/bin", "0777");
        let shared = fixture::watched_directory("/tmp", "1777");
        let ordinary = fixture::watched_directory("/usr/bin", "0755");

        assert!(FileView::new("directory|/usr/local/bin", &open).writable_by_anyone());
        assert!(
            !FileView::new("directory|/tmp", &shared).writable_by_anyone(),
            "the sticky bit is what makes a shared directory safe to share, and /tmp has \
             carried it on every unix since the eighties"
        );
        assert!(!FileView::new("directory|/usr/bin", &ordinary).writable_by_anyone());
    }

    #[test]
    fn a_mode_that_could_not_be_read_answers_nothing_rather_than_zero() {
        let mut unread = fixture::watched_file("/etc/hosts", "0644", "a9");
        unread["mode"] = Value::Null;
        let view = FileView::new("file|/etc/hosts", &unread);

        assert_eq!(view.bits(), None);
        assert!(!view.runs_as_its_owner());
        assert!(!view.writable_by_anyone());
    }
}
