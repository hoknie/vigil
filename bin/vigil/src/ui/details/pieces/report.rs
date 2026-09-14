use ratatui::text::{Line, Span};
use vigil_view::Piece;

use crate::ui::helpers::finding::acts::{self, Acts};
use crate::ui::helpers::finding::suppression;
use crate::ui::helpers::layout::{field, section, wrap};
use crate::ui::{Look, Report};

const NAME_WIDTH: usize = 9;

pub fn report(pieces: &[Piece], acts: Acts, at: Option<usize>, look: Look, width: usize) -> Report {
    let mut report = Report::default();

    for piece in pieces {
        match piece {
            Piece::Title { lead, text } => report.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(lead.clone(), look.palette.accent()),
                Span::styled(
                    match lead.is_empty() {
                        true => text.clone(),
                        false => format!("  {text}"),
                    },
                    look.palette.heading(),
                ),
            ])),
            Piece::Heading(text) => report.push(section::rule(look, text, width)),
            Piece::Field { name, value } => {
                for line in field::lines(look, name, value, NAME_WIDTH, width) {
                    report.push(line);
                }
            }
            Piece::Text(text) => {
                for part in wrap::wrap(text, width.saturating_sub(5)) {
                    report.push(Line::styled(format!("   {part}"), look.palette.quiet()));
                }
            }
            Piece::Warning(text) => {
                for part in wrap::wrap(text, width.saturating_sub(5)) {
                    report.push(Line::styled(format!("   {part}"), look.palette.alarm()));
                }
            }
            Piece::Key(key) => {
                for line in acts::lines(acts, at, look, width) {
                    report.push(line);
                }
                if !acts.buttons().is_empty() {
                    report.blank();
                }
                report.push(section::rule(look, "SUPPRESS", width));
                for line in suppression::entry(key, None) {
                    report.push(Line::raw(format!("   {line}")));
                }
                if suppression::widest(key, None) + 3 > width {
                    report.push(Line::styled(
                        "   widen the terminal before copying the key above".to_string(),
                        look.palette.alarm(),
                    ));
                }
            }
            Piece::Blank => report.blank(),
        }
    }

    report
}
