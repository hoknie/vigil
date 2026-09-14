use std::path::Path;

use vigil_config::{Edit, Entry, Suppression, silenced, write};

use super::done::Done;
use super::options::Options;
use super::shape::{known, moment};

pub const DEFAULT_PATH: &str = "/etc/vigil/vigil.yaml";

pub const RESTART: &str = "systemctl try-restart vigild.service";

pub fn add(options: &Options) -> Result<Done, String> {
    let entries = asked(options)?;
    let text = read(&options.path)?;

    match vigil_config::add(&text, &entries) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(Done {
            said: vec![format!(
                "already in {}, so nothing was written: {}",
                options.path,
                entries
                    .iter()
                    .map(|entry| match &entry.kind {
                        Some(kind) => format!("{} ({kind})", entry.key),
                        None => entry.key.clone(),
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            )],
            entries: 0,
        }),
        Edit::Changed { text: after, .. } => {
            let mut said = vec![put(&options.path, &text, &after, options.dry_run)?];
            for entry in &entries {
                said.push(format!("{}: {}", entry.named(), entry.key));
            }
            said.push(restart_note());
            Ok(Done {
                said,
                entries: entries.len(),
            })
        }
    }
}

pub fn remove(options: &Options) -> Result<Done, String> {
    let text = read(&options.path)?;

    match vigil_config::remove(&text, &options.keys) {
        Edit::NotOurs(why) => Err(why),
        Edit::AlreadySo => Ok(Done {
            said: vec![
                format!(
                    "nothing in {} is written down for {}, so the file was not touched",
                    options.path,
                    options.keys.join(", ")
                ),
                match vigil_config::named(&text) {
                    held if held.is_empty() => "it silences nothing at all".to_string(),
                    held => format!("it silences: {}", held.join(", ")),
                },
            ],
            entries: 0,
        }),
        Edit::Changed {
            text: after,
            entries,
        } => Ok(Done {
            said: vec![
                put(&options.path, &text, &after, options.dry_run)?,
                format!(
                    "{entries} entry(ies) taken out, and {} is reported again",
                    options.keys.join(", ")
                ),
                restart_note(),
            ],
            entries,
        }),
    }
}

pub fn list(options: &Options) -> Result<Done, String> {
    let held = silenced(&read(&options.path)?).map_err(|why| format!("{}: {why}", options.path))?;

    if held.is_empty() {
        return Ok(Done {
            said: vec![format!(
                "{} silences nothing: every finding this host raises reaches the console",
                options.path
            )],
            entries: 0,
        });
    }

    let mut said = vec![format!("{} suppression(s) in {}", held.len(), options.path)];
    said.extend(held.iter().map(Suppression::describe));
    Ok(Done {
        entries: held.len(),
        said,
    })
}

fn restart_note() -> String {
    format!("the daemon reads its configuration at start: {RESTART}")
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

fn put(path: &str, before: &str, after: &str, dry_run: bool) -> Result<String, String> {
    if dry_run {
        println!("{after}");
        return Ok(format!("nothing was written to {path} (--dry-run)"));
    }

    let file = Path::new(path);
    let written = write(file, after, true)?;
    silenced(after).map_err(|why| {
        let _ = std::fs::write(file, before);
        format!("what this command wrote would not load, so the file was put back: {why}")
    })?;

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
