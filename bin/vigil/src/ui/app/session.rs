use std::cell::Cell;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::Rect;

use crate::cli::{Console, Opening};

use super::App;
use crate::link::Link;

use crate::ui::{Audience, Chooser, Filter, Level, Look, Nav, Palette, Screen, View};

const REFRESH: Duration = Duration::from_secs(2);
const TICK: Duration = Duration::from_millis(250);

impl App {
    pub fn new(options: &Console, opening: Opening, palette: Palette, audience: Audience) -> Self {
        let at = opening.screen;
        let mut app = App {
            link: Link::new(options.socket.clone()),
            look: Look::new(palette, audience),
            nav: Nav::opening(at),
            view: View::nothing_yet(options.socket.clone()),
            filter: Filter::default(),
            detail_open: opening.difference,
            level: Level::default(),
            gone: None,
            chooser: Chooser::default(),
            sorting: Default::default(),
            paper: None,
            button: 0,
            in_the_buttons: false,
            helping: false,
            message: None,
            body: Cell::new(Rect::ZERO),
            refresh_wanted: false,
            leaving: false,
        };
        app.level = match opening.difference {
            true => Level::Detail,
            false => Level::top(app.rungs()),
        };
        if at != Screen::HOME {
            app.remember_section(at);
        }
        app
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut last = Instant::now();
        self.refresh();

        while !self.leaving {
            terminal.draw(|frame| frame.render_widget(self.page(), frame.area()))?;

            if event::poll(TICK)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key.code, key.modifiers);
            }

            if self.refresh_wanted || last.elapsed() >= REFRESH {
                self.refresh();
                last = Instant::now();
            }
        }

        Ok(())
    }

    pub fn answered(&self) -> bool {
        self.view.answered()
    }
}
