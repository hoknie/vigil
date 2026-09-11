use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::programs::{Showing, render};
use crate::ui::{Arrows, Program, Search, View, fixture};

pub(super) fn drawn(view: &View, program: Program, width: u16) -> String {
    drawn_with(view, program, &Search::default(), width)
}

pub(super) fn drawn_with(view: &View, program: Program, search: &Search, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 30));
    render(
        view,
        fixture::look(),
        &Showing {
            program,
            search,
            cursor: 0,
            elsewhere: 0,
            arrows: Arrows::List,
        },
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

pub(super) fn looking_for(word: &str) -> Search {
    let mut search = Search::default();
    search.start();
    for character in word.chars() {
        search.type_character(character);
    }
    search.accept();
    search
}
