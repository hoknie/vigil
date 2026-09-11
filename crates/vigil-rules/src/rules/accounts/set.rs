use super::{
    AccountUnlocked, NewAccount, PasswordChanged, PrivilegedGroupMemberAdded, RemovedAccount,
    SecondRootAccount, SudoGrantAdded,
};
use crate::RuleSet;
use crate::rules::keys::{SshKeyAdded, SshKeyRemoved};

pub fn account_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(SecondRootAccount),
        Box::new(SudoGrantAdded),
        Box::new(PrivilegedGroupMemberAdded),
        Box::new(SshKeyAdded),
        Box::new(AccountUnlocked),
        Box::new(NewAccount),
        Box::new(PasswordChanged),
        Box::new(RemovedAccount),
        Box::new(SshKeyRemoved),
    ])
}
