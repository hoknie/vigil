use serde::Deserialize;

use super::mount::Mount;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Devices {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl Devices {
    pub fn joined(&self, other: &Devices) -> Devices {
        Devices {
            include: union(&self.include, &other.include),
            exclude: union(&self.exclude, &other.exclude),
        }
    }

    pub fn allows(&self, mount: &Mount) -> bool {
        if mount.pseudo() {
            return false;
        }
        if self.exclude.iter().any(|entry| mount.named_by(entry)) {
            return false;
        }
        self.include.is_empty() || self.include.iter().any(|entry| mount.named_by(entry))
    }

    pub fn check(&self) -> Result<(), String> {
        for (list, entries) in [("include", &self.include), ("exclude", &self.exclude)] {
            if entries.iter().any(|entry| entry.trim().is_empty()) {
                return Err(format!(
                    "devices.{list}: an entry with nothing in it names no filesystem; name a \
                     device (/dev/sda1), a mount point (/mnt/nfs) or a kind of filesystem (nfs)"
                ));
            }
        }
        Ok(())
    }
}

fn union(first: &[String], second: &[String]) -> Vec<String> {
    let mut joined = first.to_vec();
    for entry in second {
        if !joined.contains(entry) {
            joined.push(entry.clone());
        }
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mounted(point: &str, filesystem: &str, source: &str) -> Mount {
        Mount {
            device: (8, 1),
            mount_point: point.to_string(),
            filesystem: filesystem.to_string(),
            source: source.to_string(),
        }
    }

    fn devices(include: &[&str], exclude: &[&str]) -> Devices {
        Devices {
            include: include.iter().map(|entry| entry.to_string()).collect(),
            exclude: exclude.iter().map(|entry| entry.to_string()).collect(),
        }
    }

    #[test]
    fn nothing_included_walks_every_filesystem_that_holds_files() {
        let every = Devices::default();

        assert!(every.allows(&mounted("/", "ext4", "/dev/sda1")));
        assert!(every.allows(&mounted("/mnt/nfs", "nfs4", "server:/export")));
        assert!(!every.allows(&mounted("/proc", "proc", "proc")));
    }

    #[test]
    fn an_included_filesystem_is_walked_and_every_other_one_is_not() {
        let only = devices(&["/dev/sda1"], &[]);

        assert!(only.allows(&mounted("/", "ext4", "/dev/sda1")));
        assert!(!only.allows(&mounted("/srv", "ext4", "/dev/sdb1")));
    }

    #[test]
    fn an_excluded_filesystem_is_not_walked_even_when_it_is_also_included() {
        let both = devices(&["nfs4", "ext4"], &["/mnt/nfs"]);

        assert!(!both.allows(&mounted("/mnt/nfs", "nfs4", "server:/export")));
        assert!(both.allows(&mounted("/mnt/other", "nfs4", "server:/other")));
    }

    #[test]
    fn a_pseudo_filesystem_is_never_walked_whatever_the_lists_say() {
        assert!(!devices(&["proc"], &[]).allows(&mounted("/proc", "proc", "proc")));
    }

    #[test]
    fn the_devices_of_a_watch_list_are_added_to_the_devices_of_the_block_and_replace_none() {
        let block = devices(&["ext4"], &["/mnt/backup"]);
        let list = devices(&["xfs", "ext4"], &["nfs"]);

        assert_eq!(
            block.joined(&list),
            devices(&["ext4", "xfs"], &["/mnt/backup", "nfs"])
        );
    }

    #[test]
    fn an_empty_entry_is_refused_rather_than_read_as_a_filesystem_nobody_has() {
        let refusal = devices(&[""], &[])
            .check()
            .expect_err("must not be accepted");

        assert!(refusal.contains("devices.include"), "{refusal}");
        assert!(devices(&["nfs"], &["/dev/sdb1"]).check().is_ok());
    }
}
