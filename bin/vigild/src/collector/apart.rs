use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use vigil_config::{Switched, blocks, is_a_directory, new_block, switched, with_block};

use super::run::Options;
use crate::config;

pub fn collectors_at(options: &Options, text: &str) -> Result<Option<PathBuf>, String> {
    vigil_config::collectors_path(Path::new(&options.path), text)
        .map_err(|cause| format!("{}: {cause}", options.path))
}

pub fn switch(options: &Options, name: &str, at: &Path, enabled: bool) -> Result<String, String> {
    let written = blocks(at)?;
    let Some(block) = written.iter().find(|block| block.name == name) else {
        return match enabled {
            false => Ok(format!(
                "{name} has no block in {}, so it is already off",
                at.display()
            )),
            true => added(options, name, at),
        };
    };

    let before = read(&block.file)?;
    match switched(&before, name, enabled) {
        Switched::NotOurs(why) => Err(format!("{}: {why}", block.file.display())),
        Switched::AlreadySo => Ok(format!(
            "{name} is already {} in {}",
            match enabled {
                true => "on",
                false => "off",
            },
            block.file.display()
        )),
        Switched::Changed(after) => config::put_beside(
            &options.path,
            &block.file,
            Some(&before),
            &after,
            options.dry_run,
        ),
    }
}

fn added(options: &Options, name: &str, at: &Path) -> Result<String, String> {
    let every_seconds = crate::modules::every_seconds_of(name).unwrap_or(60);
    let block = new_block(name, every_seconds);
    let file = match is_a_directory(at) {
        true => at.join(format!("{name}.yaml")),
        false => at.to_path_buf(),
    };

    let before = match std::fs::read_to_string(&file) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(format!("{}: {error}", file.display())),
    };
    let after = match &before {
        Some(text) => with_block(text, &block),
        None => block,
    };

    config::put_beside(
        &options.path,
        &file,
        before.as_deref(),
        &after,
        options.dry_run,
    )
}

fn read(file: &Path) -> Result<String, String> {
    std::fs::read_to_string(file).map_err(|error| format!("{}: {error}", file.display()))
}
