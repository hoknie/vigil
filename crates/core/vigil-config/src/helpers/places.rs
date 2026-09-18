use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

const READ: &[&str] = &["yaml", "yml"];

pub fn resolved(configuration: &Path, written: &str) -> PathBuf {
    let path = Path::new(written);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match configuration.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => directory.join(path),
        _ => path.to_path_buf(),
    }
}

pub fn pointed_at(configuration: &Path, text: &str, key: &str) -> Result<Option<PathBuf>, String> {
    let document: serde_yaml::Value =
        serde_yaml::from_str(text).map_err(|error| error.to_string())?;

    match document.get(key) {
        None => Ok(None),
        Some(serde_yaml::Value::Null) => Ok(None),
        Some(serde_yaml::Value::String(written)) if written.trim().is_empty() => Err(format!(
            "{key} is empty; name a file or a directory, or leave the key out"
        )),
        Some(serde_yaml::Value::String(written)) => Ok(Some(resolved(configuration, written))),
        Some(_) => Err(format!(
            "{key} is a path to a file or a directory, written as text"
        )),
    }
}

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

pub fn files_in(path: &Path) -> Result<Vec<PathBuf>, String> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    if !metadata.is_dir() {
        return Ok(vec![path.to_path_buf()]);
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
        let file = entry.path();
        if !read_here(&file) {
            continue;
        }
        if fs::metadata(&file).is_ok_and(|metadata| metadata.is_file()) {
            files.push(file);
        }
    }
    files.sort();
    Ok(files)
}

pub fn is_a_directory(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_dir(),
        Err(_) => !read_here(path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigil-places-{name}-{}-{}",
            std::process::id(),
            NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temp dir");
        directory
    }

    #[test]
    fn a_relative_path_is_read_beside_the_configuration_that_names_it() {
        assert_eq!(
            resolved(Path::new("/etc/vigil/vigil.yaml"), "suppressions"),
            PathBuf::from("/etc/vigil/suppressions")
        );
        assert_eq!(
            resolved(Path::new("/etc/vigil/vigil.yaml"), "/srv/silences"),
            PathBuf::from("/srv/silences")
        );
        assert_eq!(
            resolved(Path::new("vigil.yaml"), "suppressions"),
            PathBuf::from("suppressions")
        );
    }

    #[test]
    fn a_configuration_that_names_no_path_points_nowhere_and_that_is_not_an_error() {
        let configuration = Path::new("/etc/vigil/vigil.yaml");

        for text in ["state_dir: /var/lib/vigil\n", "", "suppressions_path:\n"] {
            assert_eq!(
                pointed_at(configuration, text, "suppressions_path"),
                Ok(None),
                "{text:?}"
            );
        }
        assert_eq!(
            pointed_at(
                configuration,
                "suppressions_path: /etc/vigil/suppressions\n",
                "suppressions_path"
            ),
            Ok(Some(PathBuf::from("/etc/vigil/suppressions")))
        );
    }

    #[test]
    fn a_path_written_as_nothing_or_as_a_list_is_refused_rather_than_read_as_none() {
        let configuration = Path::new("/etc/vigil/vigil.yaml");

        for text in ["suppressions_path: \"\"\n", "suppressions_path: [a, b]\n"] {
            assert!(
                pointed_at(configuration, text, "suppressions_path").is_err(),
                "{text:?}"
            );
        }
    }

    #[test]
    fn a_directory_is_read_file_by_file_in_the_order_of_their_names() {
        let directory = temporary("order");
        for name in ["20-deploy.yaml", "10-base.yml", "console.yaml"] {
            fs::write(directory.join(name), "suppressions: []\n").expect("writes");
        }

        let names: Vec<String> = files_in(&directory)
            .expect("reads")
            .iter()
            .map(|file| {
                file.file_name()
                    .expect("a name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        assert_eq!(names, ["10-base.yml", "20-deploy.yaml", "console.yaml"]);
    }

    #[test]
    fn what_an_editor_or_this_product_leaves_beside_a_file_is_not_read_as_one() {
        let directory = temporary("leftovers");
        for name in [
            "console.yaml.previous",
            "console.yaml.writing",
            ".console.yaml.swp",
            ".hidden.yaml",
            "console.yaml~",
            "README",
            "notes.txt",
        ] {
            fs::write(directory.join(name), "not: yaml: at: all\n").expect("writes");
        }
        fs::create_dir_all(directory.join("nested.yaml")).expect("a directory with a file's name");
        fs::write(directory.join("kept.yaml"), "suppressions: []\n").expect("writes");

        let read = files_in(&directory).expect("reads");

        assert_eq!(read, vec![directory.join("kept.yaml")]);
    }

    #[test]
    fn a_path_that_is_not_there_holds_nothing_and_is_not_an_error() {
        let directory = temporary("absent");

        assert_eq!(files_in(&directory.join("suppressions")), Ok(Vec::new()));
    }

    #[test]
    fn a_path_that_names_one_file_is_that_file_whatever_it_is_called() {
        let directory = temporary("one");
        let file = directory.join("silences.conf");
        fs::write(&file, "suppressions: []\n").expect("writes");

        assert_eq!(files_in(&file), Ok(vec![file.clone()]));
        assert!(!is_a_directory(&file));
    }

    #[test]
    fn a_path_not_made_yet_is_a_directory_unless_it_is_named_like_a_file() {
        let directory = temporary("unmade");

        assert!(is_a_directory(&directory.join("suppressions")));
        assert!(!is_a_directory(&directory.join("suppressions.yaml")));
    }
}
