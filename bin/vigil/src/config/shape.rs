use vigil_model::KnownKind;

pub fn known(named: &str) -> Result<String, String> {
    if KnownKind::parse(named).is_some() {
        return Ok(named.to_string());
    }
    let family = named.split('.').next().unwrap_or(named);
    let near: Vec<&str> = KnownKind::ALL
        .iter()
        .map(|kind| kind.as_str())
        .filter(|kind| kind.starts_with(family))
        .collect();

    Err(match near.is_empty() {
        true => format!(
            "no rule in this build raises {named:?}, so an entry naming it would silence \
             nothing. This build raises {} kinds, and the console draws the kind of every \
             finding on the screen",
            KnownKind::ALL.len()
        ),
        false => format!(
            "no rule in this build raises {named:?}. Nearest: {}",
            near.join(", ")
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_kind_no_rule_in_this_build_raises_is_refused_before_anything_is_written() {
        let refused = known("port.listen.newish").expect_err("must not be accepted");

        assert!(refused.contains("port.listen.newish"), "{refused}");
        assert!(
            refused.contains("port.listen.new"),
            "the nearest ones are named, so a typo is one word away from fixed: {refused}"
        );
    }

    #[test]
    fn a_kind_this_build_does_raise_is_taken_as_written() {
        for kind in KnownKind::ALL.iter().take(8) {
            assert_eq!(known(kind.as_str()), Ok(kind.as_str().to_string()));
        }
    }

    #[test]
    fn a_kind_from_no_family_at_all_says_where_the_kinds_are_read_off() {
        let refused = known("nonsense").expect_err("must not be accepted");

        assert!(refused.contains("draws the kind"), "{refused}");
    }
}
