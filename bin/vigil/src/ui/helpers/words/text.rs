use ratatui::buffer::Buffer;
pub fn to_text(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut lines: Vec<String> = Vec::with_capacity(area.height as usize);

    for y in area.top()..area.bottom() {
        let mut line = String::with_capacity(area.width as usize);
        for x in area.left()..area.right() {
            if let Some(cell) = buffer.cell((x, y)) {
                line.push_str(cell.symbol());
            }
        }
        lines.push(line.trim_end().to_string());
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;

    #[test]
    fn a_drawn_page_comes_back_without_the_padding_under_it() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 6));
        Paragraph::new("one\ntwo").render(buffer.area, &mut buffer);

        assert_eq!(to_text(&buffer), "one\ntwo");
    }
}
