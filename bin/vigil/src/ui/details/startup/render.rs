use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::file::file;
use super::job::job;
use super::lines::silencing;
use super::module::module;
use super::timer::timer;
use super::unit::unit;
use super::unknown::unknown;
use crate::ui::helpers::layout::page;
use crate::ui::screens::startup::{Kind, Row};
use crate::ui::types::focus::anchor;
use crate::ui::{Look, Notice, Report};

pub fn render(row: Option<&Row<'_>>, look: Look, top: usize, area: Rect, buffer: &mut Buffer) {
    let Some(row) = row else {
        Notice::plain("Nothing is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return;
    };

    let report = report(row, look, look.text_width(area.width));
    page::render(&report, look, top, area, buffer);
}

pub fn height(row: Option<&Row<'_>>, look: Look, width: usize) -> usize {
    match row {
        Some(row) => report(row, look, width).len(),
        None => 0,
    }
}

fn report(row: &Row<'_>, look: Look, width: usize) -> Report {
    let mut report = Report::default();
    match row.kind {
        Kind::Unit => unit(&mut report, row, look, width),
        Kind::Timer => timer(&mut report, row, look, width),
        Kind::Cron => job(&mut report, row, look, width),
        Kind::Module | Kind::ModulesUnreadable => module(&mut report, row, look, width),
        Kind::Script | Kind::Preload => file(&mut report, row, look, width),
        Kind::Unknown => unknown(&mut report, row, look, width),
    }
    silencing(
        &mut report,
        &format!("{}|{}", anchor::PERSISTENCE, row.key),
        look,
        width,
    );
    report
}
