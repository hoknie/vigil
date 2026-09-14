use std::path::Path;

use vigil_config::{Edit, Entry, Suppression, silenced, write};

use super::shape::{known, moment};

pub const DEFAULT_PATH: &str = "/etc/vigil/vigil.yaml";

pub const RESTART: &str = "systemctl try-restart vigild.service";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Done {
    pub said: Vec<String>,
    pub entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub keys: Vec<String>,
    pub reason: String,
    pub kind: Option<String>,
    pub until: Option<String>,
    pub prefix: bool,
    pub path: String,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            keys: Vec::new(),
            reason: String::new(),
            kind: None,
            until: None,
            prefix: false,
            path: DEFAULT_PATH.to_string(),
            dry_run: false,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const SHIPPED: &str = "state_dir: /var/lib/vigil\nsuppressions: []\nreporters: []\n";

    fn temporary(name: &str) -> String {
        let directory = std::env::temp_dir().join(format!(
            "vigil-silence-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&directory).expect("temp dir");
        let path = directory.join(name);
        std::fs::write(&path, SHIPPED).expect("writes");
        path.to_str().expect("utf-8").to_string()
    }

    fn silencing(path: &str, key: &str) -> Options {
        Options {
            keys: vec![key.to_string()],
            reason: "the staging api, expected here".into(),
            path: path.to_string(),
            ..Options::default()
        }
    }

    #[test]
    fn an_entry_written_here_is_one_the_daemon_reads_back_and_the_rest_of_the_file_is_untouched() {
        let path = temporary("write.yaml");

        let said = add(&silencing(&path, "port.listen|tcp|0.0.0.0:4444")).expect("writes");

        let after = std::fs::read_to_string(&path).expect("readable");
        assert!(after.contains("state_dir: /var/lib/vigil"), "{after}");
        assert!(after.contains("reporters: []"), "{after}");
        let held = silenced(&after).expect("the daemon would read it");
        assert_eq!(held.len(), 1);
        assert!(held[0].covers(
            "port.listen|tcp|0.0.0.0:4444",
            "port.listen.new",
            "2026-09-13T10:00:00.000Z"
        ));
        assert!(
            said.said.iter().any(|line| line.contains(RESTART)),
            "an entry that is not read until a restart, with nobody told to restart, is an \
             operator watching the same finding come back: {said:#?}"
        );
    }

    #[test]
    fn one_object_named_twice_on_the_command_line_is_asked_for_once() {
        let path = temporary("twice.yaml");

        let done = add(&Options {
            keys: vec!["user|group|docker".into(), "user|group|docker".into()],
            ..silencing(&path, "unused")
        })
        .expect("writes");

        assert_eq!(done.entries, 1, "{:#?}", done.said);
        assert_eq!(
            std::fs::read_to_string(&path)
                .expect("readable")
                .matches("user|group|docker")
                .count(),
            1
        );
    }

    #[test]
    fn what_was_written_is_taken_out_again_and_the_file_is_the_one_it_started_as() {
        let path = temporary("round.yaml");
        add(&silencing(&path, "user|group|docker")).expect("writes");

        remove(&Options {
            keys: vec!["user|group|docker".into()],
            path: path.clone(),
            ..Options::default()
        })
        .expect("writes");

        assert_eq!(std::fs::read_to_string(&path).expect("readable"), SHIPPED);
    }

    #[test]
    fn an_entry_with_no_reason_never_reaches_the_file_because_the_daemon_would_refuse_it() {
        let path = temporary("reasonless.yaml");

        let refused = add(&Options {
            reason: "   ".into(),
            ..silencing(&path, "user|group|docker")
        })
        .expect_err("must not be accepted");

        assert!(refused.contains("reason"), "{refused}");
        assert_eq!(std::fs::read_to_string(&path).expect("readable"), SHIPPED);
    }

    #[test]
    fn a_file_that_is_not_there_says_which_one_and_what_writes_it() {
        let refused = add(&silencing("/nonexistent/vigil/vigil.yaml", "a|b"))
            .expect_err("must not be invented");

        assert!(
            refused.contains("/nonexistent/vigil/vigil.yaml"),
            "{refused}"
        );
        assert!(refused.contains("vigild configure"), "{refused}");
    }

    #[test]
    fn what_the_file_holds_is_read_back_by_the_same_words_the_console_draws() {
        let path = temporary("list.yaml");
        assert!(
            list(&Options {
                path: path.clone(),
                ..Options::default()
            })
            .expect("reads")
            .said[0]
                .contains("silences nothing")
        );

        add(&silencing(&path, "user|group|docker")).expect("writes");

        let said = list(&Options {
            path,
            ..Options::default()
        })
        .expect("reads");
        assert!(said.said[0].contains("1 suppression(s)"), "{said:#?}");
        assert!(said.said[1].contains("user|group|docker"), "{said:#?}");
        assert!(said.said[1].contains("the staging api"), "{said:#?}");
    }
}
