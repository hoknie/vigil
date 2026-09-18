use super::{
    FileAppearedOrGone, FileChanged, FilePermissionsChanged, FileSuidNew, PathWritableByAll,
};
use vigil_rules::RuleSet;

pub fn file_rules() -> RuleSet {
    RuleSet::new(
        vec![Box::new(FileAppearedOrGone)],
        vec![
            Box::new(FileSuidNew),
            Box::new(FileChanged),
            Box::new(FilePermissionsChanged),
            Box::new(PathWritableByAll),
        ],
    )
}
