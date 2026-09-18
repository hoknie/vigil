use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::types::{Devices, Size, WatchList, Watched};

const PATH: &str = "path";

const MAX_FILE_SIZE: &str = "max_file_size";

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Written {
    files: Option<Vec<Value>>,
    devices: Option<Devices>,
}

pub fn watch_list_in(text: &str) -> Result<WatchList, String> {
    let document: Value = serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    let written: Written = match document {
        Value::Null => Written::default(),
        document => serde_yaml::from_value(document).map_err(|error| error.to_string())?,
    };

    let mut files: Vec<Watched> = Vec::new();
    for (place, entry) in written.files.unwrap_or_default().into_iter().enumerate() {
        let said = |why: String| format!("files #{}: {why}", place + 1);
        let watched = entry_of(entry).map_err(said)?;
        watched.check_listed().map_err(said)?;
        if files.iter().any(|before| before.path() == watched.path()) {
            return Err(said(format!(
                "{:?} is named twice, and the second entry watches nothing the first does not",
                watched.path()
            )));
        }
        files.push(watched);
    }
    let devices = written.devices.unwrap_or_default();
    devices.check()?;

    Ok(WatchList { files, devices })
}

fn entry_of(entry: Value) -> Result<Watched, String> {
    match entry {
        Value::String(path) => Ok(Watched::of(plain(&path), None)),
        Value::Mapping(pair) => pair_of(pair),
        other => Err(format!(
            "an entry is a path, or a path with its own {MAX_FILE_SIZE}, and {other:?} is \
             neither"
        )),
    }
}

fn pair_of(mut pair: Mapping) -> Result<Watched, String> {
    let path = match pair.remove(PATH) {
        Some(Value::String(path)) => plain(&path),
        _ => {
            return Err(format!(
                "an entry written as a pair names its {PATH} as text"
            ));
        }
    };
    let size = match pair.remove(MAX_FILE_SIZE) {
        None => None,
        Some(value) => Some(
            serde_yaml::from_value::<Size>(value)
                .map_err(|error| format!("{MAX_FILE_SIZE}: {error}"))?
                .bytes(),
        ),
    };
    if let Some((key, _)) = pair.into_iter().next() {
        return Err(format!(
            "{} is not a key of an entry: an entry holds {PATH} and {MAX_FILE_SIZE}",
            key.as_str().unwrap_or("a key that is not a word")
        ));
    }
    Ok(Watched::of(path, size))
}

fn plain(path: &str) -> String {
    let trimmed = path.trim();
    match trimmed.len() > 1 {
        true => trimmed.trim_end_matches('/').to_string(),
        false => trimmed.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "\
# The files watched for a changed content.
files:
  - /etc/ssh/sshd_config
  - /etc/pam.d/
  - /etc/ssh/*.conf
  - path: /etc/ssl/certs/ca.crt
    max_file_size: 8mb
devices:
  include: []
  exclude: [nfs]
";

    #[test]
    fn a_watch_list_is_read_entry_by_entry_with_the_size_each_one_names() {
        let list = watch_list_in(SHIPPED).expect("reads");

        assert_eq!(
            list.files,
            vec![
                Watched::of("/etc/ssh/sshd_config", None),
                Watched::of("/etc/pam.d", None),
                Watched::of("/etc/ssh/*.conf", None),
                Watched::of("/etc/ssl/certs/ca.crt", Some(8 * 1024 * 1024)),
            ]
        );
        assert_eq!(list.devices.exclude, vec!["nfs".to_string()]);
    }

    #[test]
    fn a_list_with_nothing_written_in_it_yet_watches_nothing_and_is_not_an_error() {
        for empty in ["", "# nothing yet\n", "files:\n", "files: []\n"] {
            assert_eq!(watch_list_in(empty), Ok(WatchList::default()), "{empty:?}");
        }
    }

    #[test]
    fn an_entry_the_agent_could_never_read_is_refused_with_its_place_in_the_list() {
        for (written, said) in [
            ("files:\n  - etc/hosts\n", "files #1"),
            ("files:\n  - /etc/hosts\n  - /etc/hosts/\n", "named twice"),
            (
                "files:\n  - path: /etc/hosts\n    size: 4096\n",
                "size is not a key",
            ),
            (
                "files:\n  - path: /etc/hosts\n    max_file_size: 8 megabytes\n",
                "30mb",
            ),
            ("files:\n  - max_file_size: 4096\n", "names its path"),
            ("files:\n  - [a, b]\n", "neither"),
            ("files:\n  - /etc/[ab\n", "never closed"),
            ("devices:\n  include: [\"\"]\n", "devices.include"),
        ] {
            let refusal = watch_list_in(written).expect_err("must not be accepted");
            assert!(refusal.contains(said), "{written}: {refusal}");
        }
    }

    #[test]
    fn a_key_a_watch_list_does_not_hold_is_refused_rather_than_quietly_left_out() {
        for written in ["paths:\n  - /etc/hosts\n", "devices:\n  includes: [nfs]\n"] {
            assert!(watch_list_in(written).is_err(), "{written}");
        }
    }
}
