use std::collections::BTreeMap;

use crate::parsers::audit::Execution;
use vigil_collect::Presence;

pub struct LaunchReading<'a> {
    pub executions: &'a [Execution],
    pub logins: &'a BTreeMap<u32, String>,
    pub any_unnamed: bool,
    pub keep_arguments: bool,
    pub on_disk: &'a dyn Fn(&str) -> Presence,
    pub from_plugin: bool,
    pub dropped: bool,
}
