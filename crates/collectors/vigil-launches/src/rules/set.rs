use super::{
    FirstLaunchForUser, LaunchFromWritablePath, LaunchSpoolDrained, LaunchSpoolDropping,
    LaunchedBinaryMissing,
};
use vigil_rules::RuleSet;

pub fn launch_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(LaunchSpoolDropping),
        Box::new(LaunchSpoolDrained),
        Box::new(LaunchFromWritablePath),
        Box::new(LaunchedBinaryMissing),
        Box::new(FirstLaunchForUser),
    ])
}
