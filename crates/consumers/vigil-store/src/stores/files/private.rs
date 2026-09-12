use std::fs::OpenOptions;
use std::path::Path;

use crate::StoreError;

pub const OWNER_ONLY: u32 = 0o600;

pub fn create_owner_only(path: &Path, append: bool) -> Result<std::fs::File, StoreError> {
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

    let file = options
        .open(path)
        .map_err(|error| StoreError::Io(format!("{}: {error}", path.display())))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(OWNER_ONLY);
        std::fs::set_permissions(path, permissions)
            .map_err(|error| StoreError::Io(format!("{}: {error}", path.display())))?;
    }

    Ok(file)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn temporary(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "vigil-private-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn a_new_file_is_readable_by_its_owner_and_nobody_else() {
        let path = temporary("new");

        let _file = create_owner_only(&path, true).expect("creates");

        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o777, OWNER_ONLY, "mode was {:o}", mode & 0o777);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_file_that_was_already_too_open_is_narrowed_rather_than_accepted() {
        let path = temporary("loose");
        std::fs::write(&path, b"{}\n").expect("write");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("chmod");

        let _file = create_owner_only(&path, true).expect("opens");

        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o777, OWNER_ONLY, "mode was {:o}", mode & 0o777);
        let _ = std::fs::remove_file(&path);
    }
}
