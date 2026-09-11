use vigil_model::{CollectorRefusal, CollectorState};

pub struct Refusal {
    pub state: Option<CollectorState>,
    pub reason: String,
}

impl Refusal {
    pub fn told(refusal: CollectorRefusal) -> Self {
        Refusal {
            state: Some(refusal.state),
            reason: refusal.reason,
        }
    }

    pub fn answered(reason: impl Into<String>) -> Self {
        Refusal {
            state: None,
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_the_daemon_explained_keeps_the_state_and_the_words_it_used() {
        let refusal = Refusal::told(CollectorRefusal::new(
            CollectorState::Unavailable,
            "auditd is not running",
        ));

        assert_eq!(refusal.state, Some(CollectorState::Unavailable));
        assert_eq!(refusal.reason, "auditd is not running");
    }

    #[test]
    fn a_refusal_with_no_state_behind_it_is_not_a_refusal_in_a_state_this_console_invented() {
        let refusal = Refusal::answered("watching ports, not \"files\"");

        assert_eq!(refusal.state, None);
        assert_eq!(refusal.reason, "watching ports, not \"files\"");
    }
}
