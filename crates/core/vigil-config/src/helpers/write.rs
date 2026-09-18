use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const OWNER_ONLY: u32 = 0o600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub path: PathBuf,
    pub previous: Option<PathBuf>,
}

pub fn write(path: &Path, text: &str, force: bool) -> Result<Written, String> {
    if path.exists() && !force {
        return Err(format!(
            "{} already exists and has not been touched.\n  --dry-run  print what would be written, and write nothing\n  --force    replace it; what is there now is kept as {}",
            path.display(),
            previous_path(path).display()
        ));
    }

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                .map_err(|error| format!("{}: {error}", parent.display()))?;
        }
    }

    let previous = match path.exists() {
        false => None,
        true => {
            let kept = previous_path(path);
            fs::copy(path, &kept).map_err(|error| format!("{}: {error}", kept.display()))?;
            set_owner_only(&kept)?;
            Some(kept)
        }
    };

    let temporary = path.with_extension("yaml.writing");
    {
        let mut file = create_owner_only(&temporary)?;
        file.write_all(text.as_bytes())
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
    }
    fs::rename(&temporary, path).map_err(|error| format!("{}: {error}", path.display()))?;

    Ok(Written {
        path: path.to_path_buf(),
        previous,
    })
}

fn previous_path(path: &Path) -> PathBuf {
    let mut name = std::ffi::OsString::from(path.as_os_str());
    name.push(".previous");
    PathBuf::from(name)
}

fn create_owner_only(path: &Path) -> Result<fs::File, String> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(OWNER_ONLY);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    set_owner_only(path)?;
    Ok(file)
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(OWNER_ONLY))
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(not(unix))]
fn set_owner_only(_path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigil-configure-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ));
        fs::create_dir_all(&directory).expect("temp dir");
        directory.join(name)
    }

    #[test]
    fn a_file_that_is_not_there_is_written_and_is_readable_by_nobody_else() {
        let path = temporary("fresh.yaml");

        let written = write(&path, "collectors: []\n", false).expect("writes");

        assert_eq!(written.path, path);
        assert_eq!(written.previous, None);
        assert_eq!(
            fs::read_to_string(&path).expect("readable"),
            "collectors: []\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("stat").permissions().mode();
            assert_eq!(mode & 0o777, OWNER_ONLY, "mode was {:o}", mode & 0o777);
        }
    }

    #[test]
    fn a_file_that_is_already_there_is_not_touched_and_the_message_says_what_to_do() {
        let path = temporary("existing.yaml");
        fs::write(&path, "# somebody's work\n").expect("writes");

        let refusal = write(&path, "collectors: []\n", false).expect_err("must not overwrite");

        assert_eq!(
            fs::read_to_string(&path).expect("readable"),
            "# somebody's work\n",
            "the file must be exactly as it was"
        );
        assert!(refusal.contains("--force"), "{refusal}");
        assert!(refusal.contains("--dry-run"), "{refusal}");
        assert!(refusal.contains(".previous"), "{refusal}");
    }

    #[test]
    fn force_replaces_it_and_leaves_the_way_back_beside_it() {
        let path = temporary("replaced.yaml");
        fs::write(&path, "# a year of somebody's suppressions\n").expect("writes");

        let written = write(&path, "collectors: [network]\n", true).expect("writes");

        assert_eq!(
            fs::read_to_string(&path).expect("readable"),
            "collectors: [network]\n"
        );
        let kept = written.previous.expect("the previous file is kept");
        assert_eq!(
            fs::read_to_string(&kept).expect("readable"),
            "# a year of somebody's suppressions\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&kept).expect("stat").permissions().mode();
            assert_eq!(
                mode & 0o777,
                OWNER_ONLY,
                "the copy is as private as the original"
            );
        }
    }

    #[test]
    fn a_directory_this_command_has_to_create_is_created_closed() {
        let path = temporary("made-up/vigil.yaml");

        write(&path, "collectors: []\n", false).expect("writes");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(path.parent().expect("a parent"))
                .expect("stat")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700, "mode was {:o}", mode & 0o777);
        }
    }

    #[test]
    fn nothing_half_written_is_left_behind() {
        let path = temporary("clean.yaml");

        write(&path, "collectors: []\n", false).expect("writes");

        assert!(
            !path.with_extension("yaml.writing").exists(),
            "the neighbour it was written to must be gone"
        );
    }
}
