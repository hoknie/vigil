const READ_ONLY: u32 = 0x0000_0001;

const LOCAL: u32 = 0x0000_1000;

const SNAPSHOT: u32 = 0x4000_0000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mounted {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub flags: u32,
    pub device: u64,
    pub block_bytes: u64,
    pub blocks: u64,
    pub blocks_available: u64,
    pub files: u64,
    pub files_free: u64,
}

impl Mounted {
    pub fn read_only(&self) -> bool {
        self.flags & READ_ONLY != 0
    }

    pub fn local(&self) -> bool {
        self.flags & LOCAL != 0
    }

    pub fn snapshot(&self) -> bool {
        self.flags & SNAPSHOT != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mounted(flags: u32) -> Mounted {
        Mounted {
            source: "/dev/disk3s1s1".into(),
            target: "/".into(),
            kind: "apfs".into(),
            flags,
            device: 0x0100_0010,
            block_bytes: 4096,
            blocks: 100,
            blocks_available: 40,
            files: 1000,
            files_free: 900,
        }
    }

    #[test]
    fn the_flags_of_a_mount_are_read_by_the_bits_the_kernel_of_macos_sets() {
        let sealed_root = mounted(0x4000_0000 | 0x0000_4000 | 0x0000_1000 | 0x0000_0001);

        assert!(sealed_root.read_only());
        assert!(sealed_root.local());
        assert!(
            sealed_root.snapshot(),
            "the system volume of a Mac is mounted from a sealed snapshot, and a Time Machine \
             snapshot is mounted the same way"
        );
        assert!(!mounted(0x0000_1000).read_only());
        assert!(!mounted(0).local());
        assert!(!mounted(0x0000_1000).snapshot());
    }
}
