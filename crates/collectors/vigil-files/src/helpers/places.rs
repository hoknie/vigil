use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const READ: &[&str] = &["yaml", "yml"];

pub fn read_here(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    !name.starts_with('.')
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| READ.contains(&extension))
}

pub fn lists_in(path: &Path) -> Result<Option<Vec<PathBuf>>, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    if !metadata.is_dir() {
        return Ok(Some(vec![path.to_path_buf()]));
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
        let file = entry.path();
        if read_here(&file) && fs::metadata(&file).is_ok_and(|metadata| metadata.is_file()) {
            files.push(file);
        }
    }
    files.sort();
    Ok(Some(files))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigil-files-places-{name}-{}-{}",
            std::process::id(),
            NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temp dir");
        directory
    }

    #[test]
    fn a_directory_of_lists_is_read_file_by_file_in_the_order_of_their_names() {
        let directory = temporary("order");
        for name in [
            "20-web.yaml",
            "10-base.yml",
            "console.yaml",
            "notes.txt",
            ".x.yaml",
        ] {
            fs::write(directory.join(name), "files: []\n").expect("writes");
        }
        fs::write(directory.join("console.yaml.previous"), "files: []\n").expect("writes");

        let read = lists_in(&directory).expect("reads").expect("there");

        assert_eq!(
            read,
            vec![
                directory.join("10-base.yml"),
                directory.join("20-web.yaml"),
                directory.join("console.yaml"),
            ],
            "the same files the configuration reads from a directory of suppressions, so an \
             operator learns one rule for every directory this product reads"
        );
    }

    #[test]
    fn a_path_that_names_one_file_is_that_file_whatever_it_is_called() {
        let directory = temporary("one");
        let file = directory.join("watch.conf");
        fs::write(&file, "files: []\n").expect("writes");

        assert_eq!(lists_in(&file), Ok(Some(vec![file.clone()])));
    }

    #[test]
    fn a_path_that_is_not_there_is_told_apart_from_a_directory_with_no_list_in_it() {
        let directory = temporary("absent");

        assert_eq!(lists_in(&directory.join("watch_fs.yaml")), Ok(None));
        assert_eq!(lists_in(&directory), Ok(Some(Vec::new())));
    }
}
