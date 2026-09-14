use serde_json::Value;
use vigil_model::{AccountChange, Changing, Snapshot};
use vigil_view::RowKey;

use crate::helpers::row_of;
use crate::types::Kind;
use crate::views::fields::text;

pub fn delete(reading: &Snapshot, row: Option<&RowKey>) -> Result<AccountChange, String> {
    let (key, item) = row_of(reading, row, Changing::Delete)?;
    match Kind::of(key) {
        Kind::Session => {}
        Kind::SessionSource => {
            return Err(
                "this row says where logins are read from; it is not a session, and there is \
                 nothing in it to end"
                    .to_string(),
            );
        }
        _ => return Err(format!("{key} is not a session")),
    }

    let identified = text(item, "session_id").is_some_and(|id| !id.is_empty())
        || item
            .get("pid")
            .and_then(Value::as_u64)
            .is_some_and(|pid| pid > 0);
    if !identified {
        return Err(
            "the reading carries neither a logind session id nor a pid for this session, so \
             there is nothing to end it by"
                .to_string(),
        );
    }
    Ok(AccountChange::DeleteSession {
        key: key.to_string(),
    })
}
