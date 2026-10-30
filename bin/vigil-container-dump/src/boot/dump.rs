use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use vigil_engines::{Answer, Dump, Engine, FAILED, WRITER};

use super::asking::ask;
use crate::accounts::Account;
use crate::cli::Options;

const ONLY_THE_OWNER: u32 = 0o600;

const ONLY_THE_OWNERS_DIRECTORY: u32 = 0o700;

pub struct Written {
    pub at: PathBuf,
    pub dump: Dump,
}

pub fn dump(options: &Options, taken_at: &str) -> Result<Vec<Written>, String> {
    let at = Path::new(&options.directory);
    DirBuilder::new()
        .recursive(true)
        .mode(ONLY_THE_OWNERS_DIRECTORY)
        .create(at)
        .map_err(|error| format!("{}: {error}", at.display()))?;

    let mut written = Vec::new();
    for engine in &options.engines {
        let path = at.join(engine.dump_file());
        let mut dump = asked(*engine, options, taken_at, at);
        dump.previous_taken_at = previous_moment(&path);
        put(&path, &dump)?;
        written.push(Written { at: path, dump });
    }

    Ok(written)
}

fn asked(engine: Engine, options: &Options, taken_at: &str, at: &Path) -> Dump {
    let Some(program) = found(engine) else {
        return Dump::absent(
            engine.name(),
            taken_at,
            options.deadline_seconds,
            format!(
                "{} is not on this host (looked in {})",
                engine.name(),
                engine.places().join(", ")
            ),
        );
    };

    let mut dump = Dump::present(engine.name(), taken_at, options.deadline_seconds, program);
    dump.written_by = WRITER.to_string();

    let account = match account_for(program) {
        Ok(account) => account,
        Err(refusal) => return refused(dump, engine, &refusal),
    };
    dump.account = account.as_ref().map(|account| account.name.clone());

    for asks in engine.asks() {
        let answer = ask(
            program,
            asks,
            Duration::from_secs(options.deadline_seconds),
            options.ceiling,
            at,
            account.as_ref(),
        );
        dump.asked.insert(asks.subject.as_str().to_string(), answer);
    }

    dump
}

#[cfg(target_os = "macos")]
fn account_for(program: &str) -> Result<Option<Account>, String> {
    crate::accounts::run_as(program).map(Some)
}

#[cfg(not(target_os = "macos"))]
fn account_for(_program: &str) -> Result<Option<Account>, String> {
    Ok(None)
}

fn refused(mut dump: Dump, engine: Engine, refusal: &str) -> Dump {
    for asks in engine.asks() {
        dump.asked.insert(
            asks.subject.as_str().to_string(),
            Answer {
                state: FAILED.to_string(),
                arguments: asks
                    .arguments
                    .iter()
                    .map(|word| (*word).to_string())
                    .collect(),
                milliseconds: 0,
                printed: String::new(),
                truncated: false,
                status: None,
                why: Some(format!("not run: {refusal}")),
            },
        );
    }
    dump
}

fn found(engine: Engine) -> Option<&'static str> {
    engine
        .places()
        .iter()
        .copied()
        .find(|place| Path::new(place).exists())
}

fn previous_moment(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let previous: Dump = serde_json::from_slice(&bytes).ok()?;
    Some(previous.taken_at)
}

fn put(path: &Path, dump: &Dump) -> Result<(), String> {
    let written = serde_json::to_vec_pretty(dump).map_err(|error| error.to_string())?;
    let temporary = path.with_extension("json.writing");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(ONLY_THE_OWNER)
            .open(&temporary)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.write_all(&written)
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.write_all(b"\n")
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
    }

    fs::rename(&temporary, path).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn workspace(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "vigil-container-dump-previous-{}-{named}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&at);
        at
    }

    fn options(at: &Path) -> Options {
        Options {
            directory: at.display().to_string(),
            engines: vec![Engine::Docker],
            ..Options::default()
        }
    }

    fn previous_of(written: &[Written]) -> Option<String> {
        written[0].dump.previous_taken_at.clone()
    }

    #[test]
    fn a_first_dump_names_no_previous_moment() {
        let at = workspace("first");

        let written = dump(&options(&at), "2026-09-17T09:00:00.000Z").expect("writes");

        assert_eq!(previous_of(&written), None);
        let _ = fs::remove_dir_all(&at);
    }

    #[test]
    fn the_second_dump_names_the_moment_of_the_first() {
        let at = workspace("second");
        dump(&options(&at), "2026-09-17T09:00:00.000Z").expect("writes");

        let written = dump(&options(&at), "2026-09-17T09:10:00.000Z").expect("writes");

        assert_eq!(
            previous_of(&written).as_deref(),
            Some("2026-09-17T09:00:00.000Z"),
            "the period the agent judges the dump by is the one the dumps show, not one \
             written down in a second place"
        );
        let on_disk: Dump =
            serde_json::from_slice(&fs::read(at.join("docker.json")).expect("the document"))
                .expect("parses");
        assert_eq!(on_disk.previous_taken_at, previous_of(&written));
        let _ = fs::remove_dir_all(&at);
    }

    #[test]
    fn a_torn_previous_dump_is_not_a_previous_moment() {
        let at = workspace("torn");
        fs::create_dir_all(&at).expect("a directory");
        fs::write(
            at.join("docker.json"),
            b"{\"engine\":\"docker\",\"taken_at\":\"2026-09",
        )
        .expect("tears");

        let written = dump(&options(&at), "2026-09-17T09:10:00.000Z").expect("writes");

        assert_eq!(previous_of(&written), None);
        let _ = fs::remove_dir_all(&at);
    }
}
