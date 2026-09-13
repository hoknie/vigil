use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

pub const OWNER_ONLY: u32 = 0o600;

pub fn create_owner_only(path: &Path, append: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(OWNER_ONLY);
    }

    let file = options.open(path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(OWNER_ONLY))?;
    }

    Ok(file)
}

#[cfg(unix)]
pub fn owner_only_directory(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
}

#[cfg(not(unix))]
pub fn owner_only_directory(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    #[test]
    fn a_spool_is_readable_by_its_owner_and_nobody_else_however_it_was_created() {
        let path = std::env::temp_dir().join(format!(
            "vigil-spool-private-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::write(&path, b"type=SYSCALL\n").expect("writes");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        let _file = create_owner_only(&path, true).expect("opens");

        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o777, OWNER_ONLY, "mode was {:o}", mode & 0o777);
        let _ = std::fs::remove_file(&path);
    }
}
