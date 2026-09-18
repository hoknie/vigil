use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use vigil_config::{
    Block, Edit, Watch, blocks, blocks_in, collectors_path, listed, paths, put, put_listed, stop,
    stop_listed, written_to,
};
use vigil_files::{Files, Watched, Watching, watch_list_in, watching_in};
use vigil_module::{Module, Settings};

const FILES: &str = "files";

const WATCHED_PATH: &str = "watched_path";

const PATHS: &str = "paths";

pub(super) enum Place {
    Configuration(PathBuf),
    Block(PathBuf),
    List(PathBuf),
}

pub(super) struct Found {
    pub(super) place: Place,
    pub(super) off: Option<PathBuf>,
}

impl Place {
    pub(super) fn of(configuration: &str) -> Result<Found, String> {
        let at = Path::new(configuration);
        let Ok(text) = fs::read_to_string(at) else {
            return Ok(Found::on(Place::Configuration(at.to_path_buf())));
        };
        let Some(collectors) =
            collectors_path(at, &text).map_err(|cause| format!("{configuration}: {cause}"))?
        else {
            return Ok(Found::on(Place::Configuration(at.to_path_buf())));
        };

        let every = blocks(&collectors)?;
        let block = every
            .iter()
            .find(|block| block.name == FILES)
            .ok_or_else(|| {
                format!(
                    "{} holds no {FILES} block, so the files collector does not run on this host \
                 and there is no watch list to write into: `vigild collector {FILES} enable` \
                 writes one",
                    collectors.display()
                )
            })?;
        let off = (!block.enabled).then(|| block.file.clone());

        Ok(Found {
            place: Place::named_by(block)?,
            off,
        })
    }

    fn named_by(block: &Block) -> Result<Place, String> {
        if block.settings.contains_key(PATHS) {
            return Ok(Place::Block(block.file.clone()));
        }
        let watched_path = match block.settings.get(WATCHED_PATH) {
            Some(written) => written.as_str().ok_or_else(|| {
                format!(
                    "{}: {WATCHED_PATH} is a path to a file or a directory, written as text",
                    block.file.display()
                )
            })?,
            None => {
                return Err(format!(
                    "the {FILES} block in {} names no {WATCHED_PATH}, so the agent watches the \
                     paths this product ships and there is no watch list to write into: add \
                     {WATCHED_PATH}: /etc/vigil/watch_fs.yaml to it",
                    block.file.display()
                ));
            }
        };
        if !watched_path.starts_with('/') {
            return Err(format!(
                "{}: {WATCHED_PATH} {watched_path:?} is not an absolute path, and the agent \
                 would refuse it",
                block.file.display()
            ));
        }
        Ok(Place::List(written_to(Path::new(watched_path), None)?))
    }

    pub(super) fn file(&self) -> &Path {
        match self {
            Place::Configuration(file) | Place::Block(file) | Place::List(file) => file,
        }
    }

    pub(super) fn read(&self) -> Result<String, String> {
        match fs::read_to_string(self.file()) {
            Ok(text) => Ok(text),
            Err(error) if error.kind() == ErrorKind::NotFound && self.is_a_list() => {
                Ok(String::new())
            }
            Err(error) => Err(format!(
                "{}: {error}. `vigild configure` writes one this console can edit",
                self.file().display()
            )),
        }
    }

    pub(super) fn named(&self, text: &str) -> Vec<Watch> {
        match self {
            Place::List(_) => listed(text),
            _ => paths(text),
        }
    }

    pub(super) fn put(&self, text: &str, entry: &Watch) -> Edit {
        match self {
            Place::List(_) => put_listed(text, entry),
            _ => put(text, entry),
        }
    }

    pub(super) fn stop(&self, text: &str, path: &str) -> Edit {
        match self {
            Place::List(_) => stop_listed(text, path),
            _ => stop(text, path),
        }
    }

    pub(super) fn reads_back(&self, after: &str, entry: &Watch) -> Result<(), String> {
        let asked = Watched::of(plain(&entry.path), entry.ceiling_bytes);
        let read: Vec<Watched> = match self {
            Place::Configuration(_) => {
                let watching = watching_in(after)?;
                watching.paths.unwrap_or_default()
            }
            Place::Block(file) => in_the_block(file, after)?,
            Place::List(_) => watch_list_in(after)?.files,
        };

        match read.contains(&asked) {
            true => Ok(()),
            false => Err(format!(
                "{} is not among the {} path(s) the agent would read from it",
                entry.path,
                read.len()
            )),
        }
    }

    pub(super) fn sized(&self, entry: &Watch) -> String {
        match (entry.ceiling_bytes, self.is_a_list()) {
            (Some(bytes), _) => format!("hashed up to {bytes} bytes"),
            (None, true) => "hashed up to the max_file_size the files block names".to_string(),
            (None, false) => "hashed up to the ceiling the files block names".to_string(),
        }
    }

    pub(super) fn applied(&self) -> String {
        match self {
            Place::List(_) => "the agent reads the watch list again at its next reading of the \
                               files, so no restart is needed; a list it cannot read is watched \
                               as it last read, and said so in the health of the collector"
                .to_string(),
            _ => "the agent takes the watched paths from this file again on its next round, so \
                  no restart is needed; a file it would not start from is left unread and said \
                  so in its log"
                .to_string(),
        }
    }

    fn is_a_list(&self) -> bool {
        matches!(self, Place::List(_))
    }
}

impl Found {
    fn on(place: Place) -> Found {
        Found { place, off: None }
    }
}

fn in_the_block(file: &Path, after: &str) -> Result<Vec<Watched>, String> {
    let written = blocks_in(file, after)?;
    let block = written
        .iter()
        .find(|block| block.name == FILES)
        .ok_or_else(|| format!("{} would hold no {FILES} block", file.display()))?;
    let said = serde_json::to_value(&block.settings).map_err(|error| error.to_string())?;
    let settings = Settings::of(String::new, FILES, said);

    Files.check(&settings)?;
    let watching: Watching = settings.read().map_err(|refusal| refusal.to_string())?;
    Ok(watching.paths.unwrap_or_default())
}

fn plain(path: &str) -> String {
    match path.len() > 1 {
        true => path.trim_end_matches('/').to_string(),
        false => path.to_string(),
    }
}
