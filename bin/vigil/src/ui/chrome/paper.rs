use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use crate::ui::helpers::layout::wrap;
use crate::ui::{Look, Paper};

const WIDEST: u16 = 76;

pub fn render(paper: &Paper, look: Look, area: Rect, buffer: &mut Buffer) {
    let width = WIDEST.min(area.width);
    let lines = laid_out(paper, width);
    let height = (lines.len() as u16 + 2).min(area.height);
    let panel = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    Clear.render(panel, buffer);

    let block = Block::bordered()
        .border_style(look.palette.border())
        .title(Line::styled(
            format!(" {} ", paper.caption),
            look.palette.heading(),
        ))
        .title_bottom(
            Line::styled(paper.footing(), look.palette.quiet()).alignment(Alignment::Right),
        );
    let inside = block.inner(panel);
    block.render(panel, buffer);

    Paragraph::new(
        lines
            .into_iter()
            .map(|line| Line::raw(format!(" {line}")))
            .collect::<Vec<Line>>(),
    )
    .render(inside, buffer);
}

fn laid_out(paper: &Paper, width: u16) -> Vec<String> {
    let room = (width as usize).saturating_sub(3);
    let mut lines = Vec::new();
    for line in &paper.lines {
        if paper.copyable || line.chars().count() <= room {
            lines.push(line.clone());
            continue;
        }
        lines.extend(wrap::wrap(line, room));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(paper: &Paper) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        render(paper, fixture::look(), buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_block_of_configuration_is_drawn_line_for_line_and_never_reflowed() {
        let long = "  - finding_key: \"tcp|0.0.0.0:4444\"  # a line longer than this panel is wide, on purpose";
        let page = drawn(&Paper::of("SUPPRESSIONS", vec![long.to_string()]).for_copying());

        assert!(
            page.contains("- finding_key: \"tcp|0.0.0.0:4444\""),
            "yaml that the console rewrapped is yaml that no longer parses when it is \
             pasted back: {page}"
        );
    }

    #[test]
    fn a_report_in_prose_is_wrapped_to_the_panel_rather_than_cut_off() {
        let said = "The agent signalled the process holding this socket and the reading taken \
                    afterwards will say whether the port is still open.";
        let page = drawn(&Paper::of("WHAT THE AGENT DID", vec![said.to_string()]));

        assert!(page.contains("The agent signalled"), "{page}");
        assert!(page.contains("afterwards"), "{page}");
        for line in page.lines() {
            assert!(line.chars().count() <= 80, "{line}");
        }
    }
}
