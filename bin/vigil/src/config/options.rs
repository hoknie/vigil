use super::silence::DEFAULT_PATH;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub keys: Vec<String>,
    pub reason: String,
    pub kind: Option<String>,
    pub until: Option<String>,
    pub prefix: bool,
    pub path: String,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            keys: Vec::new(),
            reason: String::new(),
            kind: None,
            until: None,
            prefix: false,
            path: DEFAULT_PATH.to_string(),
            dry_run: false,
        }
    }
}
