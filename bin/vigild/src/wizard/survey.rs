use vigil_collect::Health;

#[derive(Debug, Clone)]
pub struct Surveyed {
    pub name: String,
    pub subject: String,
    pub health: Health,
}

impl Surveyed {
    pub fn runs_here(&self) -> bool {
        !matches!(self.health, Health::Unavailable(_))
    }

    pub fn state(&self) -> &'static str {
        match self.health {
            Health::Ok => "ok",
            Health::Degraded(_) => "degraded",
            Health::Unavailable(_) => "unavailable",
        }
    }

    pub fn reason(&self) -> Option<&str> {
        match &self.health {
            Health::Ok => None,
            Health::Degraded(detail) | Health::Unavailable(detail) => Some(detail),
        }
    }
}

pub fn take(config: &crate::Config) -> Result<Vec<Surveyed>, String> {
    Ok(crate::boot::families(config)?
        .into_iter()
        .map(|family| Surveyed {
            name: family.collector.name().to_string(),
            subject: crate::modules::subject_of(family.collector.name())
                .unwrap_or("what it watches")
                .to_string(),
            health: family.collector.available(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_collector_that_sees_less_than_it_should_is_still_switched_on() {
        let degraded = Surveyed {
            name: "launches".into(),
            subject: "what people run".into(),
            health: Health::Degraded("the audit rule is not loaded".into()),
        };

        assert!(degraded.runs_here());
        assert_eq!(degraded.state(), "degraded");
        assert_eq!(degraded.reason(), Some("the audit rule is not loaded"));
    }

    #[test]
    fn a_collector_that_cannot_run_here_at_all_is_left_out() {
        let absent = Surveyed {
            name: "launches".into(),
            subject: "what people run".into(),
            health: Health::Unavailable("auditd is not running".into()),
        };

        assert!(!absent.runs_here());
        assert_eq!(absent.state(), "unavailable");
    }
}
