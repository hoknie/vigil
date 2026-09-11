use serde_json::Value;

pub enum What<'a> {
    Socket(&'a Value),
    Program {
        path: String,
        count: usize,
        ambiguous: bool,
    },
    Unresolved {
        count: usize,
    },
}
