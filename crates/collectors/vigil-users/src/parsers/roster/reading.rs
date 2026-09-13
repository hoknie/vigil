use std::collections::BTreeMap;

use super::key_file::UserKeyFile;
use crate::parsers::group::GroupEntry;
use crate::parsers::sessions::{Session, SessionSource};
use crate::parsers::shadow::ShadowFacts;
use crate::parsers::sudoers::SudoGrant;
use vigil_collect::PasswdEntry;

pub struct AccountsReading<'a> {
    pub passwd: &'a [PasswdEntry],
    pub groups: &'a [GroupEntry],
    pub shadow: Option<&'a BTreeMap<String, ShadowFacts>>,
    pub sudo: &'a [SudoGrant],
    pub keys: &'a [UserKeyFile],
    pub sessions: &'a [Session],
    pub session_sources: &'a [SessionSource],
}
