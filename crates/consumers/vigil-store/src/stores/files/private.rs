use std::fs::OpenOptions;
use std::path::Path;

use crate::StoreError;

pub const OWNER_ONLY: u32 = 0o600;

pub const OWNER_ONLY_DIRECTORY: u32 = 0o700;

pub fn create_owner_only_directory(path: &Path) -> Result<(), StoreError> {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(OWNER_ONLY_DIRECTORY);
    }
    builder
        .create(path)
        .map_err(|error| StoreError::Io(format!("{}: {error}", path.display())))
}

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
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "vigil-private-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ))
    }

    #[test]
    fn a_directory_this_store_makes_is_entered_by_its_owner_and_nobody_else_at_every_level() {
        let top = temporary("directory");
        let deepest = top.join("findings").join("baselines");

        create_owner_only_directory(&deepest).expect("creates");

        for made in [&top, &top.join("findings"), &deepest] {
            let mode = std::fs::metadata(made).expect("stat").permissions().mode();
            assert_eq!(
                mode & 0o777,
                OWNER_ONLY_DIRECTORY,
                "{} was {:o}: a host installed from the binaries alone has no package to make \
                 the state directory, and the store is what makes it",
                made.display(),
                mode & 0o777
            );
        }
        let _ = std::fs::remove_dir_all(&top);
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
