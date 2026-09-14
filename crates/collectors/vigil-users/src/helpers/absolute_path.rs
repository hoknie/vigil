pub fn absolute_path(what: &str, value: &str) -> Result<(), String> {
    if !value.starts_with('/') {
        return Err(format!(
            "{what} {value:?} is not an absolute path: it has to start with /"
        ));
    }
    if value.contains(':') || value.contains('\n') {
        return Err(format!(
            "{what} cannot hold a colon or a line break: /etc/passwd separates its fields with \
             colons, one account to a line"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_that_does_not_start_at_the_root_is_refused_and_the_refusal_says_what_it_needs() {
        let said = absolute_path("the shell", "bin/sh").expect_err("relative");
        assert!(said.contains("start with /"), "{said}");
        assert!(absolute_path("the shell", "/bin/sh").is_ok());
    }

    #[test]
    fn a_colon_in_a_path_would_split_the_passwd_line_and_is_refused() {
        assert!(absolute_path("the home directory", "/home/a:b").is_err());
        assert!(absolute_path("the home directory", "/home/a\nroot").is_err());
    }
}
