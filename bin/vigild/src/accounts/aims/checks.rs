use vigil_users::parse_authorized_keys;

const HIDDEN: &str = "[redacted]";

const LONGEST_NAME: usize = 32;

pub fn name(what: &str, value: &str) -> Result<(), String> {
    let body = value.strip_suffix('$').unwrap_or(value);
    let mut characters = body.chars();
    let starts = characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first == '_');
    let continues = characters.all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || "_.-".contains(character)
    });

    match starts && continues && body.len() <= LONGEST_NAME {
        true => Ok(()),
        false => Err(format!(
            "{value:?} is not a {what} name this agent writes: a lowercase letter or _, then \
             lowercase letters, digits, _ . or -, at most {LONGEST_NAME} characters"
        )),
    }
}

pub fn principal(value: &str) -> Result<(), String> {
    match value.strip_prefix('%') {
        Some(group) => name("group", group),
        None => name("account", value),
    }
}

pub fn path(what: &str, value: &str) -> Result<(), String> {
    one_line(what, value)?;
    if !value.starts_with('/') {
        return Err(format!("the {what} {value:?} is not an absolute path"));
    }
    if value.contains(':') {
        return Err(format!(
            "the {what} {value:?} holds a colon, which is the separator of /etc/passwd"
        ));
    }
    Ok(())
}

pub fn text(what: &str, value: &str) -> Result<(), String> {
    one_line(what, value)?;
    match value.contains(':') {
        true => Err(format!(
            "the {what} {value:?} holds a colon, which is the separator of /etc/passwd"
        )),
        false => Ok(()),
    }
}

pub fn one_line(what: &str, value: &str) -> Result<(), String> {
    match value.contains(['\n', '\r', '\0']) {
        true => Err(format!(
            "the {what} runs over more than one line, and a second line is a second entry in \
             the file it is written to"
        )),
        false => Ok(()),
    }
}

pub fn rule(value: &str) -> Result<(), String> {
    one_line("sudo rule", value)?;
    if value.trim().is_empty() {
        return Err("a sudo rule is empty".to_string());
    }
    if !value.contains('=') {
        return Err(format!(
            "{value:?} is not a sudo rule: a rule names hosts and commands, as in ALL=(ALL) ALL"
        ));
    }
    if value.contains(HIDDEN) {
        return Err(format!(
            "{value:?} holds {HIDDEN}: the reading hid part of this rule, and writing it back \
             would write the mark instead of what was there"
        ));
    }
    if value.trim_end().ends_with('\\') {
        return Err(format!(
            "{value:?} ends with a backslash, which joins it to the line after it"
        ));
    }
    Ok(())
}

pub fn key_line(value: &str) -> Result<String, String> {
    one_line("key", value)?;
    let keys = parse_authorized_keys(value);
    match keys.as_slice() {
        [one] => Ok(one.fingerprint.clone()),
        [] => Err(
            "the key line is not a key: it names no algorithm with a key after it that decodes"
                .to_string(),
        ),
        _ => Err("the key line holds more than one key".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB2xUXJ7lFTDnPTk1YuHnRvzTZ7nJRPWTZKGHzAtqjRr person@laptop";

    #[test]
    fn a_name_the_account_tools_would_take_is_taken_and_one_a_shell_would_read_twice_is_not() {
        for fine in ["deploy", "_apt", "svc-runner", "a.b", "machine$"] {
            assert!(name("account", fine).is_ok(), "{fine}");
        }
        for wrong in [
            "",
            "Root",
            "-rf",
            "deploy;id",
            "de ploy",
            "1abc",
            &"a".repeat(33),
        ] {
            assert!(name("account", wrong).is_err(), "{wrong}");
        }
    }

    #[test]
    fn a_sudo_principal_is_an_account_or_a_group_with_its_percent_sign() {
        assert!(principal("deploy").is_ok());
        assert!(principal("%wheel").is_ok());
        assert!(principal("%").is_err());
        assert!(principal("ALL").is_err());
    }

    #[test]
    fn a_path_is_absolute_one_line_and_free_of_the_passwd_separator() {
        assert!(path("shell", "/bin/bash").is_ok());
        for wrong in ["bin/bash", "/bin/bash\n", "/home/a:b", "/tmp/\0x"] {
            assert!(path("shell", wrong).is_err(), "{wrong:?}");
        }
    }

    #[test]
    fn a_rule_the_reading_hid_part_of_is_refused_rather_than_written_back_with_the_mark_in_it() {
        assert!(rule("ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart app").is_ok());
        assert!(rule("ALL=(ALL) /usr/bin/mysql -p[redacted]").is_err());
        assert!(rule("/usr/bin/id").is_err(), "no = is not a rule");
        assert!(rule("ALL=(ALL) ALL\ndeploy ALL=(ALL) ALL").is_err());
        assert!(rule("ALL=(ALL) ALL \\").is_err());
    }

    #[test]
    fn a_key_line_is_exactly_one_key_and_its_fingerprint_comes_back() {
        let fingerprint = key_line(KEY).expect("one key");

        assert!(fingerprint.starts_with("SHA256:"), "{fingerprint}");
        assert!(key_line("ssh-ed25519 not-base64").is_err());
        assert!(key_line(&format!("{KEY}\n{KEY}")).is_err());
    }
}
