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

pub fn moment(written: &str) -> Result<String, String> {
    let refusal = || {
        format!(
            "until: {written:?} is not a moment this agent can compare with. Write \
             2026-12-31 or 2026-12-31T00:00:00.000Z"
        )
    };

    let (date, rest) = match written.split_once('T') {
        None => (written, "00:00:00.000"),
        Some((date, time)) => (date, time.trim_end_matches('Z')),
    };
    if !shaped(date, "0000-00-00") {
        return Err(refusal());
    }
    let time = match rest.len() {
        8 => format!("{rest}.000"),
        _ => rest.to_string(),
    };
    if !shaped(&time, "00:00:00.000") {
        return Err(refusal());
    }
    Ok(format!("{date}T{time}Z"))
}

fn shaped(text: &str, like: &str) -> bool {
    text.len() == like.len()
        && text
            .chars()
            .zip(like.chars())
            .all(|(had, wanted)| match wanted {
                '0' => had.is_ascii_digit(),
                other => had == other,
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

    #[test]
    fn a_date_on_its_own_becomes_the_moment_that_day_begins() {
        assert_eq!(
            moment("2026-12-31"),
            Ok("2026-12-31T00:00:00.000Z".to_string())
        );
    }

    #[test]
    fn the_shape_the_agent_writes_its_own_moments_in_is_taken_unchanged() {
        assert_eq!(
            moment("2026-12-31T09:30:00.000Z"),
            Ok("2026-12-31T09:30:00.000Z".to_string())
        );
        assert_eq!(
            moment("2026-12-31T09:30:00Z"),
            Ok("2026-12-31T09:30:00.000Z".to_string()),
            "a moment without the milliseconds is a moment, and it is compared as text"
        );
    }

    #[test]
    fn a_moment_of_another_shape_is_refused_rather_than_compared_as_letters() {
        for written in ["31/12/2026", "tomorrow", "2026-12", "2026-12-31 09:30", ""] {
            let refused = moment(written).expect_err("must not be accepted");
            assert!(
                refused.contains("2026-12-31T00:00:00.000Z"),
                "the shape that works is in the refusal: {refused}"
            );
        }
    }
}
