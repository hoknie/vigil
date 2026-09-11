use ratatui::text::Line;

#[derive(Default)]
pub struct Report {
    lines: Vec<Line<'static>>,
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
