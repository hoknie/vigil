use serde_json::Value;
use vigil_model::{Changing, Snapshot};
use vigil_view::RowKey;

use crate::types::Kind;

pub fn row_of<'a>(
    reading: &'a Snapshot,
    row: Option<&RowKey>,
    changing: Changing,
) -> Result<(&'a str, &'a Value), String> {
    let Some(row) = row else {
        return Err(format!(
            "there is no row under the cursor to {}",
            changing.as_str()
        ));
    };
    match reading.items.get_key_value(&row.key) {
        Some((key, item)) => Ok((key.as_str(), item)),
        None => Err(format!(
            "{} is not in the reading the console holds: it may be gone since. Refresh and look \
             again.",
            row.key
        )),
    }
}

pub fn expect_kind(key: &str, kind: Kind, thing: &str) -> Result<(), String> {
    match Kind::of(key) == kind {
        true => Ok(()),
        false => Err(format!("{key} is not {thing}")),
    }
}
