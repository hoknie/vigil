use super::{NewRootProcess, ProcessBinaryDeleted, ProcessFromWritablePath, UnexpectedParent};
use crate::RuleSet;

pub fn process_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(UnexpectedParent),
        Box::new(ProcessFromWritablePath),
        Box::new(ProcessBinaryDeleted),
        Box::new(NewRootProcess),
    ])
}
