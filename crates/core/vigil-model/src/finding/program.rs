use serde_json::Value;

use crate::Finding;

impl Finding {
    pub fn program(&self) -> Option<&str> {
        if let Some(executable) = self.subject.key.get("executable").and_then(Value::as_str) {
            return Some(executable);
        }
        let running = match &self.after {
            Some(after) => after,
            None => self.before.as_ref()?,
        };
        running.pointer("/process/exe").and_then(Value::as_str)
    }
}
