use ratatui::text::Line;

#[derive(Default)]
pub struct Report {
    lines: Vec<Line<'static>>,
    buttons: Vec<(usize, usize, u16, u16)>,
}

impl Report {
    pub fn push(&mut self, line: Line<'static>) {
        self.lines.push(line);
    }

    pub fn blank(&mut self) {
        self.lines.push(Line::raw(""));
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn lines(&self) -> &[Line<'static>] {
        &self.lines
    }

    pub fn push_buttons(
        &mut self,
        lines: Vec<Line<'static>>,
        places: Vec<(usize, usize, u16, u16)>,
    ) {
        let first = self.lines.len();
        self.buttons.extend(
            places
                .into_iter()
                .map(|(button, line, x, wide)| (button, first + line, x, wide)),
        );
        self.lines.extend(lines);
    }

    pub fn buttons(&self) -> &[(usize, usize, u16, u16)] {
        &self.buttons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_as_long_as_what_was_put_into_it() {
        let mut report = Report::default();
        report.push(Line::raw("HEADING"));
        report.push(Line::raw("a line"));
        report.blank();

        assert_eq!(report.len(), 3);
        assert_eq!(report.lines().len(), 3);
    }
}
