const DISK: &str = "disk";

const DEVICE: &str = "device";

const SOURCE: &str = "source";

const UNNAMED: &str = "unnamed";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Backing {
    Disk,
    Device,
    Source,
    Unnamed,
}

impl Backing {
    pub fn name(self) -> &'static str {
        match self {
            Backing::Disk => DISK,
            Backing::Device => DEVICE,
            Backing::Source => SOURCE,
            Backing::Unnamed => UNNAMED,
        }
    }

    pub fn of(said: &str) -> Option<Backing> {
        match said {
            DISK => Some(Backing::Disk),
            DEVICE => Some(Backing::Device),
            SOURCE => Some(Backing::Source),
            UNNAMED => Some(Backing::Unnamed),
            _ => None,
        }
    }

    pub fn means(self) -> &'static str {
        match self {
            Backing::Disk => {
                "the kernel names the disk this filesystem is written to, through the partition \
                 table or the volumes stacked on it"
            }
            Backing::Device => {
                "the kernel names the block device this filesystem is written to and says \
                 nothing about what that device is made of"
            }
            Backing::Source => {
                "this filesystem is on no block device this host names, so it is gathered under \
                 what mounted it"
            }
            Backing::Unnamed => {
                "this host names neither a block device nor a source for this filesystem, so \
                 what it shares its storage with is unknown"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_way_of_learning_what_backs_a_filesystem_is_written_down_and_read_back() {
        for backing in [
            Backing::Disk,
            Backing::Device,
            Backing::Source,
            Backing::Unnamed,
        ] {
            assert_eq!(
                Backing::of(backing.name()),
                Some(backing),
                "a reading written by one build and read by the next has to name the same \
                 thing, and a word only one side knows is a row that lands nowhere"
            );
            assert!(!backing.means().is_empty(), "{}", backing.name());
        }
        assert_eq!(Backing::of("from a later version"), None);
    }

    #[test]
    fn a_filesystem_whose_disk_is_named_is_told_apart_from_one_whose_mounter_is() {
        assert!(
            Backing::Disk < Backing::Source,
            "the disks come first, because a heading naming a disk is the one a reader is \
             looking for when two mounts fill up together"
        );
        assert!(Backing::Source < Backing::Unnamed);
    }
}
