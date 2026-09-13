use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    Boot,
    Memory,
    Filesystem,
}

const BOOT: &str = "boot";

const MEMORY: &str = "memory";

const FILESYSTEM: &str = "fs";

impl Family {
    pub fn of(key: &str) -> Option<Family> {
        match key.split('|').next()? {
            BOOT => Some(Family::Boot),
            MEMORY => Some(Family::Memory),
            FILESYSTEM => Some(Family::Filesystem),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Family::Boot => BOOT,
            Family::Memory => MEMORY,
            Family::Filesystem => "filesystem",
        }
    }
}

pub struct ResourceView<'a> {
    key: &'a str,
    value: &'a Value,
}

impl<'a> ResourceView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        ResourceView { key, value }
    }

    pub fn family(&self) -> Option<Family> {
        Family::of(self.key)
    }

    pub fn is(&self, family: Family) -> bool {
        self.family() == Some(family)
    }

    pub fn boot_id(&self) -> Option<&'a str> {
        self.value["boot_id"].as_str()
    }

    pub fn booted_at(&self) -> Option<i64> {
        self.value["booted_at"].as_i64()
    }

    pub fn mount(&self) -> &'a str {
        self.value["mount"].as_str().unwrap_or("?")
    }

    pub fn device(&self) -> &'a str {
        self.value["device"].as_str().unwrap_or("?")
    }

    pub fn kind(&self) -> &'a str {
        self.value["type"].as_str().unwrap_or("?")
    }

    pub fn read_only(&self) -> bool {
        self.value["read_only"].as_bool().unwrap_or(false)
    }

    pub fn total_bytes(&self) -> u64 {
        self.value["total_bytes"].as_u64().unwrap_or(0)
    }

    pub fn free_percent_step(&self) -> Option<u64> {
        self.value["free_percent_step"].as_u64()
    }

    pub fn free_inodes_percent_step(&self) -> Option<u64> {
        self.value["free_inodes_percent_step"].as_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::fixture as neighbours;

    #[test]
    fn an_item_of_another_collector_belongs_to_no_family_here() {
        let socket = neighbours::of_another_collector("tcp|0.0.0.0:443");

        assert_eq!(ResourceView::new("tcp|0.0.0.0:443", &socket).family(), None);
        assert_eq!(
            ResourceView::new("fw-summary|nftables", &socket).family(),
            None
        );
    }

    #[test]
    fn the_boot_the_memory_and_a_filesystem_are_three_different_things() {
        let boot = fixture::boot("1f0ec2b4-6c8a-4f2b-9c0e-0b2d4a7f5e31", 1_757_419_200);
        let memory = fixture::memory(8_232_091_648, Some(1_073_737_728));
        let filesystem = fixture::filesystem("/var", Some(35), Some(85));

        assert!(ResourceView::new("boot|current", &boot).is(Family::Boot));
        assert!(ResourceView::new("memory|summary", &memory).is(Family::Memory));
        assert!(ResourceView::new("fs|/var", &filesystem).is(Family::Filesystem));
    }

    #[test]
    fn a_filesystem_that_counts_no_inodes_answers_nothing_rather_than_zero() {
        let filesystem = fixture::filesystem("/var", Some(35), None);
        let view = ResourceView::new("fs|/var", &filesystem);

        assert_eq!(view.free_inodes_percent_step(), None);
        assert_eq!(view.free_percent_step(), Some(35));
        assert_eq!(view.mount(), "/var");
        assert_eq!(view.kind(), "ext4");
    }

    #[test]
    fn a_row_from_a_host_whose_kernel_would_not_say_which_boot_it_is_answers_nothing() {
        let unreadable = fixture::boot_unreadable();
        let view = ResourceView::new("boot|current", &unreadable);

        assert_eq!(view.boot_id(), None);
        assert_eq!(view.booted_at(), None);
    }

    #[test]
    fn each_row_of_the_reading_is_named_by_the_word_its_key_opens_with() {
        assert_eq!(Family::of("boot|current"), Some(Family::Boot));
        assert_eq!(Family::of("memory|summary"), Some(Family::Memory));
        assert_eq!(Family::of("fs|/var"), Some(Family::Filesystem));
        assert_eq!(Family::of("tcp|0.0.0.0:443"), None);
        assert_eq!(Family::of("file|/etc/hosts"), None);
    }

    #[test]
    fn the_host_is_read_before_the_filesystems_it_is_made_of() {
        let mut order = vec![Family::Filesystem, Family::Memory, Family::Boot];
        order.sort();

        assert_eq!(
            order,
            vec![Family::Boot, Family::Memory, Family::Filesystem],
            "the list opens with what the whole host is and then the parts of it, and a \
             reader who sorted by nothing sees them in that order"
        );
    }
}
