use crate::cli::default_configuration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub keys: Vec<String>,
    pub reason: String,
    pub kind: Option<String>,
    pub until: Option<String>,
    pub prefix: bool,
    pub program: Option<String>,
    pub pattern: Option<String>,
    pub path: String,
    pub file: Option<String>,
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
            program: None,
            pattern: None,
            path: default_configuration(),
            file: None,
            dry_run: false,
        }
    }
}
