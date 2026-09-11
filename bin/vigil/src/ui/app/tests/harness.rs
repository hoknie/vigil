use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use crate::cli::{Cli, Command, Console};
use crate::ui::app::App;
use crate::ui::fixture;
use crate::ui::helpers::words::text;
use crate::ui::{Audience, Level, Screen};

pub(super) fn opened(line: &[&str]) -> Console {
    use clap::Parser;

    let cli = Cli::try_parse_from(std::iter::once("vigil").chain(line.iter().copied()))
        .expect("the console's own command line");

    match cli.command {
        Command::Ui(ui) => ui.console,
        Command::Capture(console) => console,
        other => panic!("{other:?}"),
    }
}

pub(super) fn app() -> App {
    on(
        &opened(&["ui", "--socket", "/nonexistent/vigil.sock"]),
        Screen::Home,
    )
}

pub(super) fn on(options: &Console, screen: Screen) -> App {
    let mut app = App::new(
        options,
        options.opening(screen),
        fixture::monochrome(),
        Audience::Person,
    );
    app.view = fixture::view();
    drawn(&app);
    app.settle();
    app
}

pub(super) fn press(app: &mut App, code: KeyCode) {
    app.on_key(code, KeyModifiers::NONE);
}

pub(super) fn typed(app: &mut App, word: &str) {
    for character in word.chars() {
        press(app, KeyCode::Char(character));
    }
}

pub(super) fn number(screen: Screen) -> KeyCode {
    KeyCode::Char(
        char::from_digit(u32::from(screen.digit().expect("a numbered section")), 10)
            .expect("one of nine"),
    )
}

pub(super) fn into(app: &mut App, screen: Screen, width: u16, height: u16) {
    press(app, number(screen));
    drawn_at(app, width, height);
    while app.level != Level::List {
        press(app, KeyCode::Down);
    }
}

pub(super) fn drawn(app: &App) -> String {
    drawn_at(app, 80, 30)
}

pub(super) fn drawn_at(app: &App, width: u16, height: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    app.draw(buffer.area, &mut buffer);
    text::to_text(&buffer)
}
