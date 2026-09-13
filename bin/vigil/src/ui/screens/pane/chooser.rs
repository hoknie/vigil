use ratatui::text::{Line, Span};
use vigil_view::{Arrangement, Toggle};

use crate::ui::Look;

pub(super) fn line(
    look: Look,
    arrangements: &[Arrangement],
    arranged: Option<&str>,
    toggles: &[Toggle],
    hidden: &[String],
) -> Line<'static> {
    let mut spans = Vec::new();

    if !arrangements.is_empty() {
        spans.push(Span::styled(" view ", look.palette.heading()));
        for arrangement in arrangements {
            let here = arranged == Some(arrangement.name);
            let label = format!("{} {}", arrangement.key, arrangement.name);
            spans.push(Span::styled(
                match here {
                    true => format!("[{label}]"),
                    false => format!(" {label} "),
                },
                match here {
                    true => look.palette.selected(),
                    false => look.palette.quiet(),
                },
            ));
            spans.push(Span::raw(" "));
        }
    }

    if !toggles.is_empty() {
        spans.push(Span::styled(" kinds ", look.palette.heading()));
        for toggle in toggles {
            let showing = !hidden.iter().any(|name| name == toggle.name);
            spans.push(Span::styled(
                match showing {
                    true => format!("[{} {}]", toggle.key, toggle.name),
                    false => format!(" {} {} ", toggle.key, toggle.name),
                },
                match showing {
                    true => look.palette.accent(),
                    false => look.palette.quiet(),
                },
            ));
        }
        spans.push(Span::styled(
            "  a all",
            match hidden.is_empty() {
                true => look.palette.quiet(),
                false => look.palette.accent(),
            },
        ));
    }

    Line::from(spans)
}
