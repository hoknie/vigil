use std::fs;
use std::path::Path;

use serde_yaml::Value;
use vigil_config::{blocks, collectors_path};

const BLOCK: &str = "launches";

const ENABLED: &str = "enabled";

const RECORD_ARGUMENTS: &str = "record_arguments";

pub fn records_arguments(configuration: &Path) -> Result<bool, String> {
    let text = fs::read_to_string(configuration)
        .map_err(|error| format!("{}: {error}", configuration.display()))?;

    let settings = match collectors_path(configuration, &text)? {
        Some(path) => blocks(&path)?
            .into_iter()
            .find(|block| block.name == BLOCK)
            .filter(|block| block.enabled)
            .map(|block| Value::Mapping(block.settings)),
        None => {
            let document: Value = serde_yaml::from_str(&text)
                .map_err(|error| format!("{}: {error}", configuration.display()))?;
            document
                .get(BLOCK)
                .cloned()
                .filter(|block| block.get(ENABLED) != Some(&Value::Bool(false)))
        }
    };

    match settings
        .as_ref()
        .and_then(|block| block.get(RECORD_ARGUMENTS))
    {
        None => Ok(false),
        Some(Value::Bool(recorded)) => Ok(*recorded),
        Some(other) => Err(format!(
            "{BLOCK}: {RECORD_ARGUMENTS} is true or false, and {other:?} is neither"
        )),
    }
}
