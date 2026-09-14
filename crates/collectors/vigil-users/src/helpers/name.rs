const LONGEST: usize = 32;

pub fn name_accepted(what: &str, value: &str) -> Result<(), String> {
    let refused = || {
        Err(format!(
            "{what} {value:?} is not a name this host accepts: a lowercase letter or _ first, then \
             lowercase letters, digits, _ . or -, at most {LONGEST} in all"
        ))
    };
    let body = value.strip_suffix('$').unwrap_or(value);
    if body.is_empty() || value.chars().count() > LONGEST {
        return refused();
    }
    let mut characters = body.chars();
    if !characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first == '_')
    {
        return refused();
    }
    if !characters.all(|one| {
        one.is_ascii_lowercase() || one.is_ascii_digit() || matches!(one, '_' | '.' | '-')
    }) {
        return refused();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_useradd_accepts_are_accepted_here() {
        for name in ["deploy", "_apt", "svc-runner", "a.b", "machine$", "x1"] {
            assert!(name_accepted("the name", name).is_ok(), "{name}");
        }
    }

    #[test]
    fn a_name_that_would_be_an_option_a_path_or_another_field_is_refused() {
        for name in [
            "",
            "-r",
            "Root",
            "a b",
            "a:b",
            "1abc",
            "../etc",
            &"a".repeat(33),
        ] {
            assert!(name_accepted("the name", name).is_err(), "{name}");
        }
    }
}
