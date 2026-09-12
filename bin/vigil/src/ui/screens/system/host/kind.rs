#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Boot,
    Memory,
    Filesystem,
}

const BOOT: &str = "boot";

const MEMORY: &str = "memory";

const FILESYSTEM: &str = "fs";

impl Kind {
    pub fn of(key: &str) -> Option<Kind> {
        match key.split('|').next()? {
            BOOT => Some(Kind::Boot),
            MEMORY => Some(Kind::Memory),
            FILESYSTEM => Some(Kind::Filesystem),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Boot => BOOT,
            Kind::Memory => MEMORY,
            Kind::Filesystem => "filesystem",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_row_of_the_reading_is_named_by_the_word_its_key_opens_with() {
        assert_eq!(Kind::of("boot|current"), Some(Kind::Boot));
        assert_eq!(Kind::of("memory|summary"), Some(Kind::Memory));
        assert_eq!(Kind::of("fs|/var"), Some(Kind::Filesystem));
    }

    #[test]
    fn a_row_of_another_collector_is_nothing_this_screen_draws() {
        assert_eq!(Kind::of("tcp|0.0.0.0:443"), None);
        assert_eq!(Kind::of("file|/etc/hosts"), None);
    }

    #[test]
    fn the_host_is_read_before_the_filesystems_it_is_made_of() {
        let mut order = vec![Kind::Filesystem, Kind::Memory, Kind::Boot];
        order.sort();

        assert_eq!(order, vec![Kind::Boot, Kind::Memory, Kind::Filesystem]);
    }
}
