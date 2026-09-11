use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::launch::launch;
use super::lines::silencing;
use super::running::running;
use super::unknown::unknown;
use crate::ui::helpers::layout::page;
use crate::ui::screens::programs::Row;
use crate::ui::types::focus::anchor;
use crate::ui::{Look, Notice, Program, Report};

pub fn render(
    row: Option<&Row<'_>>,
    program: Program,
    look: Look,
    top: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    let Some(row) = row else {
        Notice::plain("Nothing is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let report = report(row, program, look, look.text_width(area.width));
    page::render(&report, look, top, area, buffer);
}

pub fn height(row: Option<&Row<'_>>, program: Program, look: Look, width: usize) -> usize {
    match row {
        Some(row) => report(row, program, look, width).len(),
        None => 0,
    }
}

pub fn suppression_key(row: &Row<'_>, program: Program) -> String {
    match (row.mark, program) {
        (false, Program::Running) => format!("{}|{}", anchor::PROCESSES, row.key),
        (false, Program::Launches) => row.key.clone(),
        (true, _) => row.key.clone(),
    }
}

fn report(row: &Row<'_>, program: Program, look: Look, width: usize) -> Report {
    let mut report = Report::default();
    match (row.mark, program) {
        (true, _) => unknown(&mut report, row, look, width),
        (false, Program::Running) => running(&mut report, row, look, width),
        (false, Program::Launches) => launch(&mut report, row, look, width),
    }
    silencing(&mut report, &suppression_key(row, program), look, width);
    report
}
