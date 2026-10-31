use std::path::{Path, PathBuf};

use vigil_config::{
    Edit, Entry, Installation, Naming, Source, Suppression, apart, moment, silenced, write,
};

use super::done::Done;
use super::options::Options;
use super::shape::known;

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
                        Some(kind) => format!("{} ({kind})", entry.object()),
                        None => entry.object(),
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
                said.push(written_as(entry));
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
    if options.keys.is_empty() && options.program.is_none() && options.pattern.is_none() {
        return Err(
            "name an object, --program or --match; as asked it takes nothing out".to_string(),
        );
    }
    let text = read(&options.path)?;
    let configuration = Path::new(&options.path);
    let held = vigil_config::sources(configuration, &text)?;

    let mut edits: Vec<(PathBuf, String, usize)> = Vec::new();
    for source in &held {
        let before = match source.path.as_path() == configuration {
            true => text.clone(),
            false => read_or_nothing(&source.path)?,
        };
        let edit = match (&options.program, &options.pattern) {
            (None, None) => vigil_config::remove(&before, &options.keys),
            (program, pattern) => vigil_config::remove_written(
                &before,
                &options.keys,
                program.as_deref(),
                pattern.as_deref(),
            ),
        };
        match edit {
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
                    asked_for(options)
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
        asked_for(options)
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

    let template = Entry {
        program: options.program.clone(),
        kind,
        until,
        reason: options.reason.trim().to_string(),
        ..Entry::default()
    };

    let mut entries: Vec<Entry> = Vec::new();
    match (&options.pattern, options.keys.is_empty()) {
        (Some(_), false) => {
            return Err(
                "an object is named either by its keys or by --match, not both".to_string(),
            );
        }
        (Some(_), true) if options.prefix => {
            return Err("--match already covers what --prefix would; keep one".to_string());
        }
        (Some(pattern), true) => entries.push(Entry {
            key: pattern.clone(),
            naming: Naming::Pattern,
            ..template
        }),
        (None, true) if template.program.is_some() || template.kind.is_some() => {
            entries.push(Entry {
                naming: Naming::Nothing,
                ..template
            })
        }
        (None, true) => {
            return Err(
                "name an object, --program, --match or --kind; as asked it silences nothing"
                    .to_string(),
            );
        }
        (None, false) => {
            for key in &options.keys {
                let entry = Entry {
                    key: key.clone(),
                    naming: match options.prefix {
                        true => Naming::Prefix,
                        false => Naming::Exact,
                    },
                    ..template.clone()
                };
                if !entries.contains(&entry) {
                    entries.push(entry);
                }
            }
        }
    }

    for entry in &entries {
        as_written(entry)
            .validate()
            .map_err(|why| format!("{}: {why}", entry.object()))?;
    }
    Ok(entries)
}

fn as_written(entry: &Entry) -> Suppression {
    let key = |naming: Naming| (entry.naming == naming).then(|| entry.key.clone());
    Suppression {
        finding_key: key(Naming::Exact),
        finding_key_prefix: key(Naming::Prefix),
        finding_key_match: key(Naming::Pattern),
        program: entry.program.clone(),
        kind: entry.kind.clone(),
        reason: entry.reason.clone(),
        until: entry.until.clone(),
    }
}

fn written_as(entry: &Entry) -> String {
    let mut said = match entry.naming.field() {
        Some(field) => format!("{field}: {}", entry.key),
        None => String::new(),
    };
    if let Some(program) = &entry.program {
        if !said.is_empty() {
            said.push_str(", ");
        }
        said.push_str(&format!("program: {program}"));
    }
    if said.is_empty()
        && let Some(kind) = &entry.kind
    {
        said.push_str(&format!("kind: {kind}"));
    }
    said
}

fn asked_for(options: &Options) -> String {
    let mut named: Vec<String> = options.keys.clone();
    if let Some(pattern) = &options.pattern {
        named.push(pattern.clone());
    }
    if let Some(program) = &options.program {
        named.push(format!("program {program}"));
    }
    named.join(", ")
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
