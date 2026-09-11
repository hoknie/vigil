use std::process::ExitCode;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use clap::Parser;

use crate::cli::{
    CAPTURE_MOVED, CONFIGURE_LIVES_IN_THE_DAEMON, Cli, Command, Console, the_old_shape,
};
use crate::ui::{App, Audience, Palette, Screen, to_text};

const NARROWEST: u16 = 80;

const PAGE: u16 = 400;

pub fn start(arguments: impl IntoIterator<Item = String>) -> ExitCode {
    let arguments: Vec<String> = arguments.into_iter().collect();

    if let Some(where_it_went) = the_old_shape(&arguments) {
        eprintln!("{where_it_went}");
        return ExitCode::from(2);
    }

    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return match error.use_stderr() {
                true => ExitCode::from(2),
                false => ExitCode::SUCCESS,
            };
        }
    };

    match cli.command {
        Command::Ui(ui) => match ui.once {
            true => {
                eprintln!("{CAPTURE_MOVED}");
                ExitCode::from(2)
            }
            false => interactive(&ui.console),
        },
        Command::Capture(console) => capture(&console),
        Command::Configure => {
            eprintln!("{CONFIGURE_LIVES_IN_THE_DAEMON}");
            ExitCode::from(2)
        }
    }
}

fn capture(options: &Console) -> ExitCode {
    let mut app = App::new(
        options,
        options.opening(Screen::Summary),
        Palette::from_environment(),
        Audience::Script,
    );
    app.refresh();

    let mut buffer = Buffer::empty(Rect::new(0, 0, width(), PAGE));
    app.page().render(buffer.area, &mut buffer);
    println!("{}", to_text(&buffer));

    match app.answered() {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    }
}

fn interactive(options: &Console) -> ExitCode {
    let mut app = App::new(
        options,
        options.opening(Screen::Home),
        Palette::from_environment(),
        Audience::Person,
    );

    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("vigil: this is not a terminal ({error}).");
            eprintln!("       `vigil capture` prints one page as text instead.");
            return ExitCode::FAILURE;
        }
    };

    let outcome = app.run(&mut terminal);
    ratatui::restore();

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("vigil: {error}");
            ExitCode::FAILURE
        }
    }
}

fn width() -> u16 {
    ratatui::crossterm::terminal::size()
        .map(|(columns, _)| columns)
        .unwrap_or(NARROWEST)
        .max(NARROWEST)
}
