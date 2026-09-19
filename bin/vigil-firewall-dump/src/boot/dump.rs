use std::collections::BTreeMap;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::Duration;

use vigil_firewall::{
    DUMP_FILE, FirewallDump, MACOS_WRITER, MOST_ANCHORS, PF_ANCHORS, Question, parse_pf_anchors,
};

use super::asking::ask;
use crate::cli::Options;

const ONLY_THE_OWNER: u32 = 0o600;

const ONLY_THE_OWNERS_DIRECTORY: u32 = 0o700;

pub fn dump(options: &Options, taken_at: &str) -> Result<(PathBuf, FirewallDump), String> {
    let at = Path::new(&options.directory);
    DirBuilder::new()
        .recursive(true)
        .mode(ONLY_THE_OWNERS_DIRECTORY)
        .create(at)
        .map_err(|error| format!("{}: {error}", at.display()))?;

    let deadline = Duration::from_secs(options.deadline_seconds);
    let mut asked = BTreeMap::new();
    let mut turn = 0usize;
    let mut asking = |question: &Question, asked: &mut BTreeMap<String, _>| {
        turn += 1;
        asked.insert(
            question.key.clone(),
            ask(question, deadline, options.ceiling, at, turn),
        );
    };

    for question in Question::first() {
        asking(&question, &mut asked);
    }

    let anchors = asked
        .get(PF_ANCHORS)
        .filter(|answer: &&vigil_firewall::Answer| answer.answered())
        .map(|answer| parse_pf_anchors(&answer.printed))
        .unwrap_or_default();
    for anchor in anchors.iter().take(MOST_ANCHORS) {
        for question in Question::of_anchor(anchor).unwrap_or_default() {
            asking(&question, &mut asked);
        }
    }

    let dump = FirewallDump {
        taken_at: taken_at.to_string(),
        written_by: MACOS_WRITER.to_string(),
        deadline_seconds: options.deadline_seconds,
        asked,
    };
    let path = at.join(DUMP_FILE);
    put(&path, &dump)?;
    Ok((path, dump))
}

fn put(path: &Path, dump: &FirewallDump) -> Result<(), String> {
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
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.sync_all())
            .map_err(|error| format!("{}: {error}", temporary.display()))?;
    }

    fs::rename(&temporary, path).map_err(|error| format!("{}: {error}", path.display()))
}
