use super::one_line::one_line;

const HIDDEN: &str = "[redacted]";

pub fn sudo_rule(value: &str) -> Result<(), String> {
    one_line("a sudo rule", value)?;
    if value.contains(HIDDEN) {
        return Err(format!(
            "this rule carries {HIDDEN}: the reading hid part of it, and writing it back would \
             write the mark instead of what it hid"
        ));
    }
    if !value.contains('=') {
        return Err(format!(
            "{value:?} is not a sudo rule: one reads hosts=(as whom) commands, as in \
             ALL=(ALL) ALL"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rule_with_hosts_and_commands_is_accepted() {
        assert!(sudo_rule("ALL=(ALL) NOPASSWD: /usr/bin/systemctl restart app").is_ok());
    }

    #[test]
    fn a_rule_without_an_equals_sign_or_with_a_hidden_part_is_refused() {
        assert!(sudo_rule("ALL").is_err());
        assert!(sudo_rule("ALL=(ALL) /usr/bin/mysql -p[redacted]").is_err());
        assert!(sudo_rule("ALL=(ALL) ALL\nroot ALL=(ALL) ALL").is_err());
    }
}
