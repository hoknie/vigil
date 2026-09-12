use super::{
    FirstLaunchForUser, LaunchFromWritablePath, LaunchSpoolDrained, LaunchSpoolDropping,
    LaunchedBinaryMissing,
};
use crate::RuleSet;

pub fn launch_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(LaunchSpoolDropping),
        Box::new(LaunchSpoolDrained),
        Box::new(LaunchFromWritablePath),
        Box::new(LaunchedBinaryMissing),
        Box::new(FirstLaunchForUser),
    ])
}
