use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_yaml::{Mapping, Value};

use crate::helpers::places::{files_in, pointed_at};
use crate::types::block::Block;

pub const COLLECTORS_PATH: &str = "collectors_path";

const ENABLED: &str = "enabled";

const SCHEDULE: &str = "schedule";

pub fn collectors_path(configuration: &Path, text: &str) -> Result<Option<PathBuf>, String> {
    pointed_at(configuration, text, COLLECTORS_PATH)
}

pub fn blocks(path: &Path) -> Result<Vec<Block>, String> {
    let mut blocks: Vec<Block> = Vec::new();
    for file in files_in(path)? {
        let text = std::fs::read_to_string(&file)
            .map_err(|error| format!("{}: {error}", file.display()))?;
        for block in blocks_in(&file, &text)? {
            if let Some(before) = blocks.iter().find(|held| held.name == block.name) {
                return Err(format!(
                    "{} is written in {} and again in {}; keep one",
                    block.name,
                    before.file.display(),
                    block.file.display()
                ));
            }
            blocks.push(block);
        }
    }
    Ok(blocks)
}

pub fn blocks_in(file: &Path, text: &str) -> Result<Vec<Block>, String> {
    let said = |cause: String| format!("{}: {cause}", file.display());
    let mut blocks = Vec::new();

    for document in serde_yaml::Deserializer::from_str(text) {
        let value = Value::deserialize(document).map_err(|error| said(error.to_string()))?;
        let written = match value {
            Value::Null => continue,
            Value::Mapping(written) => written,
            _ => {
                return Err(said(
                    "a file of collectors is one or more mappings of a collector to its settings"
                        .to_string(),
                ));
            }
        };
        for (name, block) in written {
            let Some(name) = name.as_str().map(str::to_string) else {
                return Err(said("a collector is named by a word".to_string()));
            };
            if blocks.iter().any(|held: &Block| held.name == name) {
                return Err(said(format!("{name} is written twice; keep one")));
            }
            blocks.push(block_of(file, name, block).map_err(said)?);
        }
    }
    Ok(blocks)
}

fn block_of(file: &Path, name: String, block: Value) -> Result<Block, String> {
    let mut settings = match block {
        Value::Null => Mapping::new(),
        Value::Mapping(settings) => settings,
        _ => {
            return Err(format!(
                "{name}: its settings are a mapping of keys to values, or nothing at all"
            ));
        }
    };

    let enabled = match settings.remove(ENABLED) {
        None => true,
        Some(Value::Bool(enabled)) => enabled,
        Some(other) => {
            return Err(format!(
                "{name}: {ENABLED} is true or false, and {other:?} is neither"
            ));
        }
    };
    let schedule = match settings.remove(SCHEDULE) {
        None => None,
        Some(value) => {
            Some(seconds(&value).map_err(|cause| format!("{name}: {SCHEDULE}: {cause}"))?)
        }
    };

    Ok(Block {
        name,
        file: file.to_path_buf(),
        enabled,
        schedule,
        settings,
    })
}

pub fn seconds(value: &Value) -> Result<u32, String> {
    let refused = || {
        format!(
            "{} is not a period: write seconds as a number, or 30s, 5m, 1h",
            match value {
                Value::String(text) => format!("{text:?}"),
                other => format!("{other:?}"),
            }
        )
    };
    let seconds = match value {
        Value::Number(number) => number.as_u64().ok_or_else(refused)?,
        Value::String(text) => {
            let text = text.trim();
            let (digits, unit) = text.split_at(
                text.find(|character: char| !character.is_ascii_digit())
                    .unwrap_or(text.len()),
            );
            let count: u64 = digits.parse().map_err(|_| refused())?;
            let by = match unit.trim() {
                "" | "s" => 1,
                "m" => 60,
                "h" => 3600,
                _ => return Err(refused()),
            };
            count.checked_mul(by).ok_or_else(refused)?
        }
        _ => return Err(refused()),
    };
    match u32::try_from(seconds) {
        Ok(0) => Err(
            "0 is not a period: a collector that reads constantly is a collector that \
                       costs the host everything"
                .to_string(),
        ),
        Ok(seconds) => Ok(seconds),
        Err(_) => Err(refused()),
    }
}
