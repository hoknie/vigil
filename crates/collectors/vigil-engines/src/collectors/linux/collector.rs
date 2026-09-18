use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::SystemTime;

use vigil_collect::{CollectError, Collector, Health};
use vigil_model::{Rfc3339, Snapshot};

use super::health::{Standing, standing};
use crate::parsers::{
    EngineReading, EnginesReading, SOURCE, engines_snapshot, parse_daemon_json,
    parse_registries_conf,
};
use crate::types::{Dump, Engine, Registry, Watching};

pub const DUMP_CEILING_BYTES: u64 = 16 * 1024 * 1024;

pub struct EnginesCollector {
    now: Box<dyn Fn() -> Rfc3339 + Send + Sync>,
    directory: PathBuf,
    watching: Watching,
    registries: Vec<(Engine, PathBuf)>,
}

pub struct Held {
    pub engine: Engine,
    pub dump: Result<Dump, CollectError>,
    pub age_seconds: Option<u64>,
    pub registries: Vec<Registry>,
    pub registries_refusal: Option<String>,
}

impl EnginesCollector {
    pub fn new(now: impl Fn() -> Rfc3339 + Send + Sync + 'static, watching: Watching) -> Self {
        EnginesCollector::with_paths(now, crate::types::DUMP_DIRECTORY, watching)
    }

    pub fn with_paths(
        now: impl Fn() -> Rfc3339 + Send + Sync + 'static,
        directory: impl Into<PathBuf>,
        watching: Watching,
    ) -> Self {
        EnginesCollector {
            now: Box::new(now),
            directory: directory.into(),
            registries: Engine::ALL
                .into_iter()
                .map(|engine| (engine, PathBuf::from(engine.registries_file())))
                .collect(),
            watching,
        }
    }

    pub fn reading_registries_from(self, from: &[(Engine, &str)]) -> Self {
        EnginesCollector {
            registries: from
                .iter()
                .map(|(engine, path)| (*engine, PathBuf::from(path)))
                .collect(),
            ..self
        }
    }

    pub fn dump_path(&self, engine: Engine) -> PathBuf {
        self.directory.join(engine.dump_file())
    }

    pub fn held(&self) -> Vec<Held> {
        self.watching
            .watched()
            .into_iter()
            .map(|engine| self.of(engine))
            .collect()
    }

    fn of(&self, engine: Engine) -> Held {
        let (dump, age_seconds) = match self.read(engine) {
            Ok((dump, age)) => (Ok(dump), age),
            Err(refusal) => (Err(refusal), None),
        };
        let (registries, registries_refusal) = self.registries_of(engine);

        Held {
            engine,
            dump,
            age_seconds,
            registries,
            registries_refusal,
        }
    }

    fn read(&self, engine: Engine) -> Result<(Dump, Option<u64>), CollectError> {
        let path = self.dump_path(engine);
        let shown = path.display().to_string();

        let metadata = fs::metadata(&path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => CollectError::Absent(shown.clone()),
            ErrorKind::PermissionDenied => CollectError::Denied(shown.clone()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;

        if metadata.len() > DUMP_CEILING_BYTES {
            return Err(CollectError::Budget(format!(
                "{shown}: {} bytes, over the {DUMP_CEILING_BYTES} this collector reads",
                metadata.len()
            )));
        }

        let bytes = fs::read(&path).map_err(|error| match error.kind() {
            ErrorKind::PermissionDenied => CollectError::Denied(shown.clone()),
            _ => CollectError::Unreadable(format!("{shown}: {error}")),
        })?;

        let dump = crate::parsers::parse_dump(&bytes)
            .map_err(|refusal| CollectError::Unreadable(format!("{shown}: {refusal}")))?;

        Ok((dump, self.age_seconds(&metadata)))
    }

    fn age_seconds(&self, metadata: &fs::Metadata) -> Option<u64> {
        let written_at = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(written_at).ok()?.as_secs();

        match age > self.watching.stale_after_seconds() {
            true => Some(age),
            false => None,
        }
    }

    fn registries_of(&self, engine: Engine) -> (Vec<Registry>, Option<String>) {
        let Some((_, path)) = self.registries.iter().find(|(named, _)| *named == engine) else {
            return (Vec::new(), None);
        };
        let shown = path.display().to_string();

        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => return (Vec::new(), None),
            Err(error) => return (Vec::new(), Some(format!("{shown}: {error}"))),
        };

        match engine {
            Engine::Docker => match parse_daemon_json(&text, &shown) {
                Ok(found) => (found, None),
                Err(refusal) => (Vec::new(), Some(refusal)),
            },
            Engine::Podman => (parse_registries_conf(&text, &shown), None),
        }
    }

    pub fn stale_after_seconds(&self) -> u64 {
        self.watching.stale_after_seconds()
    }
}

impl Collector for EnginesCollector {
    fn name(&self) -> &'static str {
        SOURCE
    }

    fn available(&self) -> Health {
        standing(self, &self.held())
    }

    fn collect(&self) -> Result<Snapshot, CollectError> {
        let held = self.held();

        if let Standing::NotOne(refusal) = Standing::of(&held) {
            return Err(refusal);
        }

        let engines: Vec<EngineReading<'_>> = held
            .iter()
            .map(|one| EngineReading {
                engine: one.engine,
                dump: one.dump.as_ref().ok(),
                registries: one.registries.clone(),
                registries_read: one.registries_refusal.is_none(),
            })
            .collect();

        Ok(engines_snapshot(
            &(self.now)(),
            &EnginesReading { engines: &engines },
        ))
    }
}
