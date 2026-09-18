use vigil_config::{Edit, Watch, write};

use super::place::{Found, Place};
use crate::config::done::Done;

pub fn watch(configuration: &str, entry: &Watch) -> Result<Done, String> {
    let Found { place, off } = Place::of(configuration)?;
    let file = place.file().display().to_string();
    let text = place.read()?;
    let named = place
        .named(&text)
        .iter()
        .any(|held| held.path == entry.path);

    match place.put(&text, entry) {
        Edit::NotOurs(why) => Err(format!("{file}: {why}")),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "{file} already watches {} exactly as asked, so nothing was written",
                entry.path
            )],
            entries: 0,
            ..Done::default()
        }),
        Edit::Changed { text: after, .. } => {
            let mut said = vec![
                written(&place, &after, entry)?,
                match named {
                    true => format!("how {} is watched changed", entry.path),
                    false => format!("{} is watched from now on", entry.path),
                },
                place.sized(entry),
                place.applied(),
            ];
            said.extend(switched_off(off));
            Ok(Done {
                said,
                entries: 1,
                ..Done::default()
            })
        }
    }
}

pub fn unwatch(configuration: &str, path: &str) -> Result<Done, String> {
    let Found { place, off } = Place::of(configuration)?;
    let file = place.file().display().to_string();
    let text = place.read()?;

    match place.stop(&text, path) {
        Edit::NotOurs(why) => Err(format!("{file}: {why}")),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "{file} does not name {path}, so the file was not touched: it is watched by \
                 nothing this console can take away"
            )],
            entries: 0,
            ..Done::default()
        }),
        Edit::Changed { text: after, .. } => {
            let mut said = vec![
                stopped(&place, &after, path)?,
                format!("{path} is no longer watched"),
                place.applied(),
            ];
            said.extend(switched_off(off));
            Ok(Done {
                said,
                entries: 1,
                ..Done::default()
            })
        }
    }
}

fn written(place: &Place, after: &str, entry: &Watch) -> Result<String, String> {
    place.reads_back(after, entry).map_err(|why| {
        format!(
            "what this console would write would not be read back by the agent, so nothing was \
             written: {why}"
        )
    })?;
    saved(place, after)
}

fn stopped(place: &Place, after: &str, path: &str) -> Result<String, String> {
    let left = place.named(after);
    for held in &left {
        place.reads_back(after, held).map_err(|why| {
            format!(
                "what this console would write would not be read back by the agent, so nothing \
                 was written: {why}"
            )
        })?;
    }
    if left.iter().any(|held| held.path == path) {
        return Err(format!(
            "{path} would still be named, so nothing was written"
        ));
    }
    saved(place, after)
}

fn saved(place: &Place, after: &str) -> Result<String, String> {
    let done = write(place.file(), after, true)?;

    Ok(match &done.previous {
        Some(previous) => format!(
            "wrote {} (0600), and what was there is kept as {}",
            done.path.display(),
            previous.display()
        ),
        None => format!("wrote {} (0600)", done.path.display()),
    })
}

fn switched_off(off: Option<std::path::PathBuf>) -> Vec<String> {
    off.map(|file| {
        format!(
            "the files collector is switched off in {}, so nothing is watched until it is \
             enabled again",
            file.display()
        )
    })
    .into_iter()
    .collect()
}
