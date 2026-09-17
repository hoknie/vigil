#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockEntry {
    pub partition_of: Option<String>,
    pub made_of: Vec<String>,
}

impl BlockEntry {
    pub fn nothing() -> BlockEntry {
        BlockEntry::default()
    }

    pub fn partition_of(disk: impl Into<String>) -> BlockEntry {
        BlockEntry {
            partition_of: Some(disk.into()),
            made_of: Vec::new(),
        }
    }

    pub fn made_of(parts: &[&str]) -> BlockEntry {
        BlockEntry {
            partition_of: None,
            made_of: parts.iter().map(|part| (*part).to_string()).collect(),
        }
    }

    pub fn says_nothing(&self) -> bool {
        self.partition_of.is_none() && self.made_of.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_whole_disk_is_a_device_the_kernel_says_nothing_further_about() {
        assert!(BlockEntry::nothing().says_nothing());
        assert!(!BlockEntry::partition_of("sda").says_nothing());
        assert!(!BlockEntry::made_of(&["sda3"]).says_nothing());
    }

    #[test]
    fn a_device_the_kernel_never_heard_of_reads_as_one_it_says_nothing_about() {
        assert_eq!(
            BlockEntry::default(),
            BlockEntry::nothing(),
            "a name that is not in /sys and a whole disk both leave this agent with no parent \
             to walk to, and reading one of them as an error would drop the filesystem out of \
             the list"
        );
    }
}
