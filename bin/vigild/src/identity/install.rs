use std::fs;
use std::path::Path;

use vigil_model::Uuid7;

use crate::helpers::uuid7;

const FILE: &str = "install_id";

pub fn install_id(state_dir: &Path) -> Result<Uuid7, String> {
    vigil_store::create_owner_only_directory(state_dir).map_err(|error| {
        format!(
            "the state directory {} cannot be created: {error}",
            state_dir.display()
        )
    })?;

    let path = state_dir.join(FILE);
    if let Ok(existing) = fs::read_to_string(&path) {
        let existing = existing.trim().to_string();
        if !existing.is_empty() {
            return Ok(existing);
        }
    }

    let minted = uuid7::mint();
    write_owner_only(&path, &format!("{minted}\n"))?;
    Ok(minted)
}

fn write_owner_only(path: &Path, contents: &str) -> Result<(), String> {
    use std::io::Write;

    let mut options = fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(path)
        .map_err(|error| format!("{} cannot be written: {error}", path.display()))?;
    file.write_all(contents.as_bytes())
        .map_err(|error| format!("{} cannot be written: {error}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("{} cannot be secured: {error}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_minted_once_and_then_read_back_for_ever() {
        let dir = std::env::temp_dir().join(format!("vigil-install-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let first = install_id(&dir).expect("mints");
        let second = install_id(&dir).expect("reads back");

        assert_eq!(first, second, "a restart must not look like a new install");
        assert_eq!(first.len(), 36);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(dir.join(FILE))
                .expect("stat")
                .permissions()
                .mode();
            assert_eq!(
                mode & 0o777,
                0o600,
                "the installation's identity is owner-only, whatever the umask was: {:o}",
                mode & 0o777
            );
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
