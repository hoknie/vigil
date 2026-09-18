use std::path::PathBuf;

use crate::types::suppression::Suppression;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub path: PathBuf,
    pub suppressions: Vec<Suppression>,
}
