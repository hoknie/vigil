use std::cell::Cell;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::layout::Rect;

use crate::terminal::capture;

use crate::cli::{Console, Opening};

use super::App;
use crate::link::Link;

use crate::ui::{
    Audience, Chooser, Dismissed, Filter, Level, Look, Nav, Palette, Picked, Pointer, Screen, View,
};

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
            picked: Picked::default(),
            dismissed: Dismissed::default(),
            asking: None,
            editing: None,
            graph: None,
            history: None,
            silences: None,
            named_configuration: options.config.clone(),
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
            cursor: Cell::new(None),
            pointer: Pointer::starting(true),
            rows_seen: Default::default(),
            listed_seen: Default::default(),
            index_seen: Default::default(),
            found_seen: Default::default(),
            tally_seen: Default::default(),
            counts_seen: Default::default(),
            detail_seen: Default::default(),
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
        let mut held = false;
        self.refresh();

        while !self.leaving {
            held = capture::follow(held, self.wants_the_mouse())?;
            terminal.draw(|frame| {
                frame.render_widget(self.page(), frame.area());
                if let Some(cursor) = self.cursor() {
                    frame.set_cursor_position(cursor);
                }
            })?;

            if event::poll(TICK)? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.on_key(key.code, key.modifiers);
                    }
                    Event::Mouse(mouse) => self.on_mouse(mouse),
                    Event::Resize(_, _) => self.resized(),
                    _ => {}
                }
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
