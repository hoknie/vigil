use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::ui::{Look, View};

pub(super) fn title(look: Look, view: &View) -> Paragraph<'static> {
    let Some(status) = &view.status else {
        return Paragraph::new(Line::styled(
            format!(" vigil · no answer from {}", view.socket_path),
            look.palette.alarm(),
        ));
    };

    Paragraph::new(Line::from(vec![
        Span::styled(" vigil ", look.palette.title()),
        Span::styled(status.host.hostname.clone(), look.palette.accent()),
        Span::raw(format!(
            "  {} {} · {} · agent {}",
            status.host.os.distro,
            status.host.os.version,
            status.host.os.arch,
            status.agent.version,
        )),
    ]))
}
