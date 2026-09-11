use vigil_model::{AgentStatus, CollectorState, Host, Rfc3339};

use super::found::Found;
use super::readings::{Reading, Readings};
use crate::link::Trouble;

pub struct View {
    pub socket_path: String,
    pub status: Option<Status>,
    pub readings: Readings,
    pub found: Found,
    pub trouble: Option<Trouble>,
}

pub struct Status {
    pub host: Host,
    pub agent: AgentStatus,
    pub sent_at: Rfc3339,
}

impl View {
    pub fn nothing_yet(socket_path: impl Into<String>) -> Self {
        View {
            socket_path: socket_path.into(),
            status: None,
            readings: Readings::default(),
            found: Found::default(),
            trouble: None,
        }
    }

    pub fn reading(&self, collector: &str) -> &Reading {
        self.readings.of(collector)
    }

    pub fn collector_state(&self, name: &str) -> Option<CollectorState> {
        let status = self.status.as_ref()?;
        status
            .agent
            .collectors
            .iter()
            .find(|collector| collector.name == name)
            .map(|collector| collector.state.clone())
    }

    pub fn collector_reason(&self, name: &str) -> Option<&str> {
        let status = self.status.as_ref()?;
        status
            .agent
            .collectors
            .iter()
            .find(|collector| collector.name == name)
            .and_then(|collector| collector.reason.as_deref())
    }

    pub fn collector_note(&self, name: &str) -> Option<&str> {
        match self.collector_state(name)? {
            CollectorState::Ok | CollectorState::Off => None,
            _ => self.collector_reason(name),
        }
    }

    pub fn switched_off(&self, name: &str) -> bool {
        self.collector_state(name) == Some(CollectorState::Off)
    }

    pub fn answered(&self) -> bool {
        self.status.is_some() && self.trouble.is_none()
    }

    pub fn has_reading(&self) -> bool {
        self.status.is_some()
    }

    pub fn stale(&self) -> Option<&Trouble> {
        match self.status.is_some() {
            true => self.trouble.as_ref(),
            false => None,
        }
    }

    pub fn as_of(&self) -> Option<&str> {
        self.status.as_ref().map(|status| status.sent_at.as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::link::{Trouble, TroubleKind};

    use crate::ui::fixture;

    #[test]
    fn a_console_with_nothing_in_it_has_not_been_answered_and_has_nothing_to_go_stale() {
        let view = super::View::nothing_yet("/run/vigil/vigil.sock");

        assert!(!view.answered());
        assert!(!view.has_reading());
        assert!(view.stale().is_none());
    }

    #[test]
    fn a_reading_kept_after_the_daemon_stopped_answering_is_stale_and_not_an_answer() {
        let mut view = fixture::view();
        view.trouble = Some(Trouble::new(
            "/run/vigil/vigil.sock",
            TroubleKind::Absent,
            "No such file or directory (os error 2)",
        ));

        assert!(!view.answered(), "nobody answered the last question");
        assert!(view.has_reading(), "there is still something to draw");
        assert!(view.stale().is_some(), "and it has to say that it is old");
    }

    #[test]
    fn a_collector_switched_off_is_told_apart_from_one_that_is_failing() {
        let view = fixture::view();

        assert!(view.switched_off("launches"));
        assert!(
            view.collector_note("launches").is_none(),
            "off is a decision, not a fault to report over a list"
        );
        assert!(
            view.collector_reason("launches").is_some(),
            "and the decision still has to name the key that reverses it"
        );
    }
}
