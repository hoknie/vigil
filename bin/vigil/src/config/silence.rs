use std::path::{Path, PathBuf};

use vigil_config::{Edit, Entry, Installation, Source, Suppression, apart, silenced, write};

use super::done::Done;
use super::options::Options;
use super::shape::{known, moment};

pub const DEFAULT_PATH: &str = Installation::here().configuration;

pub const NEXT_ROUND: &str = "the agent takes it up on its next round";

pub fn add(options: &Options) -> Result<Done, String> {
    let entries = asked(options)?;
    let text = read(&options.path)?;
    let configuration = Path::new(&options.path);
    let held = vigil_config::sources(configuration, &text)?;
    let target = target(configuration, &text, options.file.as_deref())?;

    let fresh: Vec<Entry> = entries
        .iter()
        .filter(|entry| !said_anywhere(&held, &as_written(entry)))
        .cloned()
        .collect();
    if fresh.is_empty() {
        return Ok(Done {
            said: vec![format!(
                "already silenced, so nothing was written: {} ({})",
                entries
                    .iter()
                    .map(|entry| match &entry.kind {
                        Some(kind) => format!("{} ({kind})", entry.key),
                        None => entry.key.clone(),
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
                holding(&held, &entries)
            )],
            entries: 0,
            ..Done::default()
        });
    }

    let before = match target.as_path() == configuration {
        true => text.clone(),
        false => read_or_nothing(&target)?,
    };
    match vigil_config::add(&before, &fresh) {
        Edit::NotOurs(why) => Err(format!("{}: {why}", target.display())),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "already in {}, so nothing was written",
                target.display()
            )],
            entries: 0,
            ..Done::default()
        }),
        Edit::Changed { text: after, .. } => {
            let mut said = vec![put(&target, configuration, &after, options.dry_run)?];
            for entry in &fresh {
                said.push(format!("{}: {}", entry.named(), entry.key));
            }
            said.push(restart_note());
            Ok(Done {
                said,
                entries: fresh.len(),
                file: Some(target),
            })
        }
    }
}

pub fn remove(options: &Options) -> Result<Done, String> {
    let text = read(&options.path)?;
    let configuration = Path::new(&options.path);
    let held = vigil_config::sources(configuration, &text)?;

    let mut edits: Vec<(PathBuf, String, usize)> = Vec::new();
    for source in &held {
        let before = match source.path.as_path() == configuration {
            true => text.clone(),
            false => read_or_nothing(&source.path)?,
        };
        match vigil_config::remove(&before, &options.keys) {
            Edit::NotOurs(why) => return Err(format!("{}: {why}", source.path.display())),
            Edit::AlreadySo => {}
            Edit::Changed {
                text: after,
                entries,
            } => edits.push((source.path.clone(), after, entries)),
        }
    }

    if edits.is_empty() {
        let named: Vec<String> = held
            .iter()
            .flat_map(|source| source.suppressions.iter())
            .map(Suppression::describe)
            .collect();
        return Ok(Done {
            said: vec![
                format!(
                    "nothing in {} is written down for {}, so no file was touched",
                    places(&held),
                    options.keys.join(", ")
                ),
                match named.is_empty() {
                    true => "they silence nothing at all".to_string(),
                    false => format!("they silence: {}", named.join("; ")),
                },
            ],
            entries: 0,
            ..Done::default()
        });
    }

    let mut said = Vec::new();
    let mut taken = 0;
    for (path, after, entries) in &edits {
        said.push(put(path, configuration, after, options.dry_run)?);
        taken += entries;
    }
    said.push(format!(
        "{taken} entry(ies) taken out, and {} is reported again",
        options.keys.join(", ")
    ));
    said.push(restart_note());
    Ok(Done {
        said,
        entries: taken,
        ..Done::default()
    })
}

pub fn take_out(options: &Options, from: &Path, suppression: &Suppression) -> Result<Done, String> {
    let text = read(&options.path)?;
    let configuration = Path::new(&options.path);
    let held = vigil_config::sources(configuration, &text)?;
    if !held.iter().any(|source| source.path == from) {
        return Err(format!(
            "{} is not a file {} reads suppressions from, so it was not touched",
            from.display(),
            options.path
        ));
    }

    let before = match from == configuration {
        true => text,
        false => read_or_nothing(from)?,
    };
    match vigil_config::remove_one(&before, suppression) {
        Edit::NotOurs(why) => Err(format!("{}: {why}", from.display())),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "{} no longer holds {}, so it was not touched",
                from.display(),
                suppression.describe()
            )],
            entries: 0,
            ..Done::default()
        }),
        Edit::Changed {
            text: after,
            entries,
        } => Ok(Done {
            said: vec![
                put(from, configuration, &after, options.dry_run)?,
                restart_note(),
            ],
            entries,
            file: Some(from.to_path_buf()),
        }),
    }
}

pub fn every(path: &str) -> Result<Vec<Source>, String> {
    let text = read(path)?;
    vigil_config::sources(Path::new(path), &text)
}

pub fn list(options: &Options) -> Result<Done, String> {
    let held: Vec<Source> = every(&options.path)?
        .into_iter()
        .filter(|source| !source.suppressions.is_empty())
        .collect();
    let total: usize = held.iter().map(|source| source.suppressions.len()).sum();

    if total == 0 {
        return Ok(Done {
            said: vec![format!(
                "{} silences nothing, and neither does a file it points at: every finding this \
                 host raises reaches the console",
                options.path
            )],
            entries: 0,
            ..Done::default()
        });
    }

    let mut said = vec![format!("{total} suppression(s) in {}", places(&held))];
    for source in &held {
        said.push(format!("{}:", source.path.display()));
        said.extend(
            source
                .suppressions
                .iter()
                .map(|suppression| format!("  {}", suppression.describe())),
        );
    }
    Ok(Done {
        entries: total,
        said,
        ..Done::default()
    })
}

fn target(configuration: &Path, text: &str, file: Option<&str>) -> Result<PathBuf, String> {
    match vigil_config::suppressions_path(configuration, text)
        .map_err(|why| format!("{}: {why}", configuration.display()))?
    {
        Some(path) => vigil_config::written_to(&path, file),
        None => match file {
            None => Ok(configuration.to_path_buf()),
            Some(named) => Err(format!(
                "{} names no {}, so there is no directory to put {named} in. Add \
                 `{}: /etc/vigil/suppressions` to it",
                configuration.display(),
                vigil_config::SUPPRESSIONS_PATH,
                vigil_config::SUPPRESSIONS_PATH
            )),
        },
    }
}

fn said_anywhere(held: &[Source], asked: &Suppression) -> bool {
    held.iter()
        .flat_map(|source| source.suppressions.iter())
        .any(|suppression| suppression.says_the_same_as(asked))
}

fn holding(held: &[Source], entries: &[Entry]) -> String {
    let files: Vec<String> = held
        .iter()
        .filter(|source| {
            entries.iter().any(|entry| {
                source
                    .suppressions
                    .iter()
                    .any(|suppression| suppression.says_the_same_as(&as_written(entry)))
            })
        })
        .map(|source| source.path.display().to_string())
        .collect();
    format!("in {}", files.join(", "))
}

fn places(held: &[Source]) -> String {
    held.iter()
        .map(|source| source.path.display().to_string())
        .collect::<Vec<String>>()
        .join(", ")
}

fn restart_note() -> String {
    format!(
        "{NEXT_ROUND}, with no restart; a file it would not start from is left unread and said \
         so in its log"
    )
}

fn asked(options: &Options) -> Result<Vec<Entry>, String> {
    let kind = match &options.kind {
        None => None,
        Some(named) => Some(known(named)?),
    };
    let until = match &options.until {
        None => None,
        Some(written) => Some(moment(written)?),
    };

    let mut entries: Vec<Entry> = Vec::new();
    for key in &options.keys {
        let entry = Entry {
            key: key.clone(),
            prefix: options.prefix,
            kind: kind.clone(),
            until: until.clone(),
            reason: options.reason.trim().to_string(),
        };
        if !entries.contains(&entry) {
            entries.push(entry);
        }
    }

    for entry in &entries {
        as_written(entry)
            .validate()
            .map_err(|why| format!("{}: {why}", entry.key))?;
    }
    Ok(entries)
}

fn as_written(entry: &Entry) -> Suppression {
    Suppression {
        finding_key: match entry.prefix {
            true => None,
            false => Some(entry.key.clone()),
        },
        finding_key_prefix: match entry.prefix {
            true => Some(entry.key.clone()),
            false => None,
        },
        kind: entry.kind.clone(),
        reason: entry.reason.clone(),
        until: entry.until.clone(),
    }
}

fn put(file: &Path, configuration: &Path, after: &str, dry_run: bool) -> Result<String, String> {
    let reads_back = match file == configuration {
        true => silenced(after).map(|_| ()),
        false => apart(after).map(|_| ()),
    };
    reads_back.map_err(|why| {
        format!(
            "what this command would write to {} would not load, so nothing was written: {why}",
            file.display()
        )
    })?;

    if dry_run {
        println!("{after}");
        return Ok(format!(
            "nothing was written to {} (--dry-run)",
            file.display()
        ));
    }

    let written = write(file, after, true)?;
    Ok(match &written.previous {
        Some(previous) => format!(
            "wrote {} (0600), and what was there is kept as {}",
            written.path.display(),
            previous.display()
        ),
        None => format!("wrote {} (0600)", written.path.display()),
    })
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| {
        format!("{path}: {error}. `vigild configure` writes one this command can edit")
    })
}

fn read_or_nothing(path: &Path) -> Result<String, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(format!("{}: {error}", path.display())),
    }
}
