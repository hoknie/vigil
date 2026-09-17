use std::process::ExitCode;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use clap::Parser;

use crate::cli::{
    CAPTURE_MOVED, COLLECTOR_LIVES_IN_THE_DAEMON, CONFIGURE_LIVES_IN_THE_DAEMON, Cli, Command,
    Console, Silencing, Ui, the_old_shape,
};
use crate::config;
use crate::terminal::capture;
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
            false => interactive(&ui),
        },
        Command::Capture(console) => capture(&console),
        Command::Suppress(asked) => {
            let options = asked.options();
            let (doing, outcome) = match asked.doing {
                Silencing::Add(_) => ("add", config::add(&options)),
                Silencing::Remove(_) => ("remove", config::remove(&options)),
                Silencing::List(_) => ("list", config::list(&options)),
            };
            match outcome {
                Ok(done) => {
                    eprintln!("vigil suppress {doing}:\n  {}", done.said.join("\n  "));
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("vigil suppress {doing}: {error}");
                    ExitCode::FAILURE
                }
            }
        }
        Command::Configure => {
            eprintln!("{CONFIGURE_LIVES_IN_THE_DAEMON}");
            ExitCode::from(2)
        }
        Command::Collector { .. } => {
            eprintln!("{COLLECTOR_LIVES_IN_THE_DAEMON}");
            ExitCode::from(2)
        }
    }
}

fn capture(options: &Console) -> ExitCode {
    let mut app = App::new(
        options,
        options.opening(Screen::SUMMARY),
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

fn interactive(asked: &Ui) -> ExitCode {
    let options = &asked.console;
    let mut app = App::new(
        options,
        options.opening(Screen::HOME),
        Palette::from_environment(),
        Audience::Person,
    )
    .with_the_mouse(!asked.no_mouse);

    let mut terminal = match ratatui::try_init() {
        Ok(terminal) => terminal,
        Err(error) => {
            eprintln!("vigil: this is not a terminal ({error}).");
            eprintln!("       `vigil capture` prints one page as text instead.");
            return ExitCode::FAILURE;
        }
    };

    capture::release_on_panic();
    let outcome = app.run(&mut terminal);
    capture::release();
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
