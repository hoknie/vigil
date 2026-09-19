use std::io;
use std::mem::MaybeUninit;

use crate::types::Mounted;

const ATTEMPTS: usize = 4;

const SPARE: usize = 8;

const _: () = assert!(std::mem::size_of::<libc::fsid_t>() == std::mem::size_of::<[i32; 2]>());

pub fn mounted() -> io::Result<Vec<Mounted>> {
    for _ in 0..ATTEMPTS {
        let counted = unsafe { libc::getfsstat(std::ptr::null_mut(), 0, libc::MNT_NOWAIT) };
        let Ok(counted) = usize::try_from(counted) else {
            return Err(io::Error::last_os_error());
        };

        let room = counted + SPARE;
        let mut table: Vec<MaybeUninit<libc::statfs>> = Vec::with_capacity(room);
        let bytes = room * std::mem::size_of::<libc::statfs>();
        let filled = unsafe {
            libc::getfsstat(
                table.as_mut_ptr().cast(),
                libc::c_int::try_from(bytes).unwrap_or(libc::c_int::MAX),
                libc::MNT_NOWAIT,
            )
        };
        let Ok(filled) = usize::try_from(filled) else {
            return Err(io::Error::last_os_error());
        };
        if filled == room {
            continue;
        }

        unsafe { table.set_len(filled) };
        return Ok(table
            .iter()
            .map(|entry| described(unsafe { entry.assume_init_ref() }))
            .collect());
    }

    Err(io::Error::from_raw_os_error(libc::EAGAIN))
}

fn described(entry: &libc::statfs) -> Mounted {
    Mounted {
        source: text(&entry.f_mntfromname),
        target: text(&entry.f_mntonname),
        kind: text(&entry.f_fstypename),
        flags: entry.f_flags,
        device: device_of(&entry.f_fsid),
        block_bytes: u64::from(entry.f_bsize),
        blocks: entry.f_blocks,
        blocks_available: entry.f_bavail,
        files: entry.f_files,
        files_free: entry.f_ffree,
    }
}

fn device_of(identity: &libc::fsid_t) -> u64 {
    let halves: [i32; 2] = unsafe { std::mem::transmute_copy(identity) };
    u64::from(halves[0].cast_unsigned())
}

fn text(field: &[libc::c_char]) -> String {
    let bytes: Vec<u8> = field
        .iter()
        .take_while(|character| **character != 0)
        .map(|character| character.cast_unsigned())
        .collect();
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::MetadataExt;

    use super::*;

    #[test]
    fn the_root_of_this_host_is_mounted_from_a_device_and_says_how_full_it_is() {
        let table = mounted().expect("read");

        let root = table
            .iter()
            .find(|mount| mount.target == "/")
            .expect("every Mac mounts a root");

        assert!(root.source.starts_with("/dev/disk"), "{root:?}");
        assert!(!root.kind.is_empty());
        assert!(root.local());
        assert!(root.blocks > 0 && root.block_bytes > 0);
        assert!(root.blocks_available <= root.blocks);
    }

    #[test]
    fn the_device_a_mount_names_is_the_device_every_file_on_it_carries() {
        let table = mounted().expect("read");
        let home = std::env::temp_dir();
        let device = std::fs::metadata(&home)
            .expect("the temporary directory")
            .dev();

        assert!(
            table
                .iter()
                .any(|mount| mount.device == device & u64::from(u32::MAX)),
            "a walk that steps from one filesystem into another sees it by the device number \
             of the file, and the mount table has to answer with the same number: {device:#x}"
        );
    }
}
