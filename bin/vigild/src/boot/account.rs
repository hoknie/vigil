use vigil_config::Account;

pub fn account_line(account: &Account, name: Option<&str>) -> Option<String> {
    if account.uid == 0 {
        return None;
    }
    let who = match name {
        Some(name) => format!("{name} (uid {})", account.uid),
        None => format!("uid {}", account.uid),
    };
    Some(format!(
        "  account: {who}, not root: each collector says below what it cannot read"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(uid: u32) -> Account {
        Account { uid, home: None }
    }

    #[test]
    fn a_daemon_that_is_not_root_says_under_which_account_it_runs() {
        assert_eq!(
            account_line(&account(501), Some("ann")).as_deref(),
            Some(
                "  account: ann (uid 501), not root: each collector says below what it cannot read"
            )
        );
    }

    #[test]
    fn a_daemon_run_by_root_says_nothing_about_its_account() {
        assert_eq!(account_line(&account(0), Some("root")), None);
    }

    #[test]
    fn an_account_without_a_name_is_said_by_its_uid() {
        let said = account_line(&account(501), None).expect("a line");

        assert!(said.contains("uid 501, not root"), "{said}");
        assert_eq!(said.matches("501").count(), 1);
    }
}
