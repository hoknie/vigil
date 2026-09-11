use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowFacts {
    pub password: PasswordState,
    pub last_change_day: Option<i64>,
    pub max_age_days: Option<i64>,
    pub expires_day: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordState {
    Set,
    Locked,
    Disabled,
    Empty,
}

impl PasswordState {
    pub fn as_str(self) -> &'static str {
        match self {
            PasswordState::Set => "set",
            PasswordState::Locked => "locked",
            PasswordState::Disabled => "disabled",
            PasswordState::Empty => "empty",
        }
    }

    pub fn permits_login(self) -> bool {
        matches!(self, PasswordState::Set | PasswordState::Empty)
    }
}

pub fn parse_shadow(text: &str) -> BTreeMap<String, ShadowFacts> {
    let mut accounts = BTreeMap::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 8 {
            continue;
        }

        let password = match fields[1] {
            "" => PasswordState::Empty,
            secret if secret.starts_with('!') => PasswordState::Locked,
            secret if secret.starts_with('*') => PasswordState::Disabled,
            _ => PasswordState::Set,
        };

        accounts.insert(
            fields[0].to_string(),
            ShadowFacts {
                password,
                last_change_day: fields[2].parse().ok(),
                max_age_days: fields[4].parse().ok(),
                expires_day: fields[7].parse().ok(),
            },
        );
    }

    accounts
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHADOW: &str = "\
root:$6$rounds=5000$abcdefgh$0123456789abcdefghijklmnop:19000:0:99999:7:::
daemon:*:19000:0:99999:7:::
backup:!$6$saltsalt$hashhashhash:18800:0:99999:7::19999:
openvpn::19100:0:99999:7:::
locked-twice:!!:18000:0:99999:7:::
short:line:only
";

    #[test]
    fn reads_the_state_and_the_dates_and_nothing_that_could_be_cracked() {
        let shadow = parse_shadow(SHADOW);

        assert_eq!(shadow["root"].password, PasswordState::Set);
        assert_eq!(shadow["root"].last_change_day, Some(19000));
        assert_eq!(shadow["root"].max_age_days, Some(99999));
        assert_eq!(shadow["root"].expires_day, None);
    }

    #[test]
    fn no_part_of_a_hash_survives_the_parse() {
        let printed = format!("{:?}", parse_shadow(SHADOW));

        for secret in ["$6$", "rounds=5000", "abcdefgh", "saltsalt", "hashhashhash"] {
            assert!(
                !printed.contains(secret),
                "the parsed shadow still carries {secret}: {printed}"
            );
        }
    }

    #[test]
    fn tells_locked_disabled_and_passwordless_apart() {
        let shadow = parse_shadow(SHADOW);

        assert_eq!(shadow["backup"].password, PasswordState::Locked);
        assert_eq!(shadow["locked-twice"].password, PasswordState::Locked);
        assert_eq!(shadow["daemon"].password, PasswordState::Disabled);
        assert_eq!(
            shadow["openvpn"].password,
            PasswordState::Empty,
            "an account with no password is not a locked account"
        );

        assert!(shadow["openvpn"].password.permits_login());
        assert!(!shadow["backup"].password.permits_login());
        assert!(!shadow["daemon"].password.permits_login());
    }

    #[test]
    fn an_expiry_date_survives_and_a_short_line_does_not_become_an_account() {
        let shadow = parse_shadow(SHADOW);

        assert_eq!(shadow["backup"].expires_day, Some(19999));
        assert!(!shadow.contains_key("short"));
    }
}
