use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::Look;
use crate::ui::helpers::layout::wrap;

pub struct Notice {
    headline: String,
    detail: Vec<String>,
    loud: bool,
}

impl Notice {
    pub fn plain(headline: impl Into<String>) -> Self {
        Notice {
            headline: headline.into(),
            detail: Vec::new(),
            loud: false,
        }
    }

    pub fn loud(headline: impl Into<String>) -> Self {
        Notice {
            loud: true,
            ..Notice::plain(headline)
        }
    }

    pub fn saying(mut self, sentence: impl Into<String>) -> Self {
        self.detail.push(sentence.into());
        self
    }

    pub fn lines(&self, look: Look, width: usize) -> Vec<Line<'static>> {
        let room = width.saturating_sub(5);
        let mut lines = vec![Line::styled(
            format!("   {}", self.headline),
            match self.loud {
                true => look.palette.alarm(),
                false => look.palette.heading(),
            },
        )];
        for sentence in &self.detail {
            for part in wrap::wrap(sentence, room) {
                lines.push(Line::raw(format!("   {part}")));
            }
        }
        lines
    }

    pub fn render(&self, look: Look, area: Rect, buffer: &mut Buffer) {
        Paragraph::new(self.lines(look, area.width as usize)).render(area, buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    #[test]
    fn a_notice_says_its_headline_and_every_sentence_under_it() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 8));

        Notice::loud("The agent is not answering.")
            .saying("Is the agent running? systemctl status vigild")
            .render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        assert!(page.contains("not answering"), "{page}");
        assert!(page.contains("systemctl"), "{page}");
    }

    #[test]
    fn a_sentence_wider_than_the_terminal_is_wrapped_rather_than_cut() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 8));

        Notice::plain("Short.")
            .saying("The socket belongs to root and lets nobody else in, which is the design")
            .render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        for line in page.lines() {
            assert!(line.chars().count() <= 40, "{line}");
        }
        assert!(page.contains("design"), "nothing is lost: {page}");
    }
}
