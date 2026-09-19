use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::{Look, Notice, View};

pub fn render(view: &View, look: Look, area: Rect, buffer: &mut Buffer) {
    let notice = match &view.trouble {
        Some(trouble) => {
            let mut notice = Notice::loud(trouble.headline()).saying(trouble.cause.clone());
            for step in trouble.what_to_try() {
                notice = notice.saying(format!("· {step}"));
            }
            notice
        }
        None => Notice::plain("Waiting for the agent.")
            .saying(format!("Socket: {}", view.socket_path))
            .saying("Asked again every couple of seconds; r asks now."),
    };

    notice.render(look, area, buffer);
}

#[cfg(test)]
mod tests {
    use crate::link::{Trouble, TroubleKind};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(view: &View) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 20));
        render(view, fixture::look(), buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_daemon_that_is_not_running_is_a_sentence_with_the_path_in_it() {
        let mut view = View::nothing_yet("/run/vigil/vigil.sock");
        view.trouble = Some(Trouble::new(
            "/run/vigil/vigil.sock",
            TroubleKind::Absent,
            "No such file or directory (os error 2)",
        ));

        let page = drawn(&view);

        assert!(page.contains("/run/vigil/vigil.sock"), "{page}");
        assert!(
            page.contains(vigil_config::Installation::here().service.status),
            "and what to do about it: {page}"
        );
    }

    #[test]
    fn a_socket_this_user_may_not_open_says_that_rather_than_that_nothing_is_running() {
        let mut view = View::nothing_yet("/run/vigil/vigil.sock");
        view.trouble = Some(Trouble::new(
            "/run/vigil/vigil.sock",
            TroubleKind::Forbidden,
            "Permission denied (os error 13)",
        ));

        let page = drawn(&view);

        assert!(page.contains("may not open"), "{page}");
        assert!(page.contains("sudo vigil ui"), "{page}");
    }

    #[test]
    fn a_console_that_has_only_just_opened_is_waiting_rather_than_broken() {
        let page = drawn(&View::nothing_yet("/run/vigil/vigil.sock"));

        assert!(page.contains("Waiting for the agent"), "{page}");
        assert!(page.contains("/run/vigil/vigil.sock"), "{page}");
        assert!(
            !page.contains("not answering"),
            "nobody has been asked yet: {page}"
        );
    }
}
