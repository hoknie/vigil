use super::{FileChanged, FilePermissionsChanged, FileSuidNew, PathWritableByAll};
use crate::RuleSet;

pub fn file_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(FileSuidNew),
        Box::new(FileChanged),
        Box::new(FilePermissionsChanged),
        Box::new(PathWritableByAll),
    ])
}
