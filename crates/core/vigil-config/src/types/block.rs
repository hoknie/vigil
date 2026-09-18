use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub name: String,
    pub file: PathBuf,
    pub enabled: bool,
    pub schedule: Option<u32>,
    pub settings: serde_yaml::Mapping,
}
