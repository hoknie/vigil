use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limit {
    unseen: Vec<String>,
}

impl Limit {
    pub fn of(unseen: Vec<String>) -> Option<Limit> {
        let unseen: Vec<String> = unseen
            .into_iter()
            .filter(|part| !part.trim().is_empty())
            .collect();
        match unseen.is_empty() {
            true => None,
            false => Some(Limit { unseen }),
        }
    }

    pub fn unseen(&self) -> &[String] {
        &self.unseen
    }

    pub fn said(&self) -> String {
        self.unseen.join("; ")
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.said())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_limit_is_never_empty() {
        assert_eq!(Limit::of(Vec::new()), None);
        assert_eq!(Limit::of(vec![" ".into(), String::new()]), None);
    }

    #[test]
    fn a_limit_names_each_thing_it_does_not_see_in_the_order_it_was_told() {
        let limit = Limit::of(vec![
            "the crontab of every account is not read".into(),
            String::new(),
            "the login and logout hooks are not read".into(),
        ])
        .expect("two things are not seen");

        assert_eq!(limit.unseen().len(), 2);
        assert_eq!(
            limit.to_string(),
            "the crontab of every account is not read; the login and logout hooks are not read"
        );
    }
}
