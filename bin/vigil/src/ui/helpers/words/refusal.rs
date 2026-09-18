use crate::ui::{Notice, Refusal};

pub fn refused(refusal: &Refusal, collector: &str, unread: &str) -> Notice {
    let headline = match &refusal.state {
        Some(state) => format!("This reading was refused: {collector} is {state}."),
        None => "This reading was refused.".to_string(),
    };

    Notice::loud(headline)
        .saying(refusal.reason.clone())
        .saying(unread.to_string())
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use vigil_model::{CollectorRefusal, CollectorState};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn page(refusal: &Refusal) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 10));
        refused(
            refusal,
            "launches",
            "No launch is listed here: nothing was read.",
        )
        .render(fixture::look(), buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_refusal_names_the_state_the_reason_and_what_is_missing_because_of_it() {
        let drawn = page(&Refusal::told(CollectorRefusal::new(
            CollectorState::Unavailable,
            "auditd is not running: program launches are not visible",
        )));

        assert!(drawn.contains("launches is unavailable"), "{drawn}");
        assert!(drawn.contains("auditd is not running"), "{drawn}");
        assert!(drawn.contains("No launch is listed here"), "{drawn}");
    }

    #[test]
    fn the_reason_is_the_daemon_s_own_words_and_not_a_sentence_this_console_made_up() {
        let said = "/var/log/audit/audit.log is not present on this system";

        let drawn = page(&Refusal::told(CollectorRefusal::new(
            CollectorState::Degraded,
            said,
        )));

        assert!(drawn.contains(said), "{drawn}");
    }

    #[test]
    fn a_refusal_that_came_without_a_state_does_not_get_one_put_into_its_mouth() {
        let drawn = page(&Refusal::answered("watching network, not \"launches\""));

        assert!(!drawn.contains("unavailable"), "{drawn}");
        assert!(!drawn.contains("degraded"), "{drawn}");
        assert!(drawn.contains("watching network"), "{drawn}");
    }
}
