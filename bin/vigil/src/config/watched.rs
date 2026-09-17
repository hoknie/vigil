use std::path::Path;

use vigil_config::{Edit, Watch, paths, put, stop, write};
use vigil_files::{Watched, watching_in};

use super::done::Done;

pub fn watch(file: &str, entry: &Watch) -> Result<Done, String> {
    let text = read(file)?;
    let named = paths(&text).iter().any(|held| held.path == entry.path);

    match put(&text, entry) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "{} already watches {} exactly as asked, so nothing was written",
                file, entry.path
            )],
            entries: 0,
        }),
        Edit::Changed { text: after, .. } => {
            let said = vec![
                written(file, &after, entry)?,
                match named {
                    true => format!("how {} is watched changed", entry.path),
                    false => format!("{} is watched from now on", entry.path),
                },
                hashed(entry),
                applied(),
            ];
            Ok(Done { said, entries: 1 })
        }
    }
}

pub fn unwatch(file: &str, path: &str) -> Result<Done, String> {
    let text = read(file)?;

    match stop(&text, path) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "{file} does not name {path}, so the file was not touched: it is watched by \
                 nothing this console can take away"
            )],
            entries: 0,
        }),
        Edit::Changed { text: after, .. } => {
            let said = vec![
                written(file, &after, &Watch::of(path, None))?,
                format!("{path} is no longer watched"),
                applied(),
            ];
            Ok(Done { said, entries: 1 })
        }
    }
}

fn written(file: &str, after: &str, entry: &Watch) -> Result<String, String> {
    reads_back(after, entry).map_err(|why| {
        format!(
            "what this console would write would not be read back by the agent, so nothing was \
             written: {why}"
        )
    })?;
    let done = write(Path::new(file), after, true)?;

    Ok(match &done.previous {
        Some(previous) => format!(
            "wrote {} (0600), and what was there is kept as {}",
            done.path.display(),
            previous.display()
        ),
        None => format!("wrote {} (0600)", done.path.display()),
    })
}

fn reads_back(after: &str, entry: &Watch) -> Result<(), String> {
    let watching = watching_in(after)?;
    let asked = Watched::of(entry.path.clone(), entry.ceiling_bytes);

    match watching.paths.contains(&asked) {
        true => Ok(()),
        false => Err(format!(
            "{} is not among the {} path(s) the agent would read from it",
            entry.path,
            watching.paths.len()
        )),
    }
}

fn hashed(entry: &Watch) -> String {
    match entry.ceiling_bytes {
        Some(ceiling_bytes) => format!("hashed up to {ceiling_bytes} bytes"),
        None => "hashed up to the ceiling the files block names".to_string(),
    }
}

fn applied() -> String {
    "the agent takes the watched paths from this file again on its next round, so no restart \
     is needed; a file it would not start from is left unread and said so in its log"
        .to_string()
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| {
        format!("{path}: {error}. `vigild configure` writes one this console can edit")
    })
}
