use std::collections::BTreeMap;

use super::key_file::UserKeyFile;
use crate::parsers::accounts::group::GroupEntry;
use crate::parsers::accounts::passwd::PasswdEntry;
use crate::parsers::accounts::shadow::ShadowFacts;
use crate::parsers::accounts::sudoers::SudoGrant;
use crate::parsers::accounts::utmp::Session;

pub struct AccountsReading<'a> {
    pub passwd: &'a [PasswdEntry],
    pub groups: &'a [GroupEntry],
    pub shadow: Option<&'a BTreeMap<String, ShadowFacts>>,
    pub sudo: &'a [SudoGrant],
    pub keys: &'a [UserKeyFile],
    pub sessions: Option<&'a [Session]>,
}
