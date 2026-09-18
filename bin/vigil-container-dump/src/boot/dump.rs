use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use vigil_engines::{Dump, Engine, WRITER};

use super::asking::ask;
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
        let dump = asked(*engine, options, taken_at, at);
        let path = at.join(engine.dump_file());
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

    for asks in engine.asks() {
        let answer = ask(
            program,
            asks,
            Duration::from_secs(options.deadline_seconds),
            options.ceiling,
            at,
        );
        dump.asked.insert(asks.subject.as_str().to_string(), answer);
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
