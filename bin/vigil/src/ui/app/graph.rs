use std::time::Instant;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::App;

use crate::ui::details::pieces;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::motion::keys;
use crate::ui::screens::graph::HEADER_LINES;
use crate::ui::{Action, Graph, Reading};

pub const GRAPH: char = 'P';

pub const WATCHING: char = 'w';

const NO_GRAPH_HERE: &str = "This list draws no graph of its rows: P draws the path a packet \
                             takes, on the list of interfaces in the firewall section.";

const NOTHING_UNDER_THE_CURSOR: &str = "There is no row under the cursor to draw the path of.";

const NOT_READ_YET: &str = "This list has not been read yet, so there is no path to draw.";

impl App {
    pub(in crate::ui::app) fn open_the_graph(&mut self) -> bool {
        let Some(pane) = self.pane() else {
            return false;
        };
        if !pane.offers().graph {
            self.message = Some(NO_GRAPH_HERE.to_string());
            return false;
        }
        let row = match self.pane_row_under_the_cursor() {
            Some(row) if row.of_the_reading => row,
            _ => {
                self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
                return false;
            }
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            self.message = Some(NOT_READ_YET.to_string());
            return false;
        };

        let drawn = pane.graph(snapshot, &row);
        if drawn.is_empty() {
            self.message = Some(NOTHING_UNDER_THE_CURSOR.to_string());
            return false;
        }
        self.graph = Some(Graph::open(row, drawn));
        self.chooser.close();
        false
    }

    pub(super) fn walk_the_graph(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match keys::action(code, modifiers, false) {
            Action::Leave => self.leaving = true,
            Action::Back | Action::Sideways(-1) => {
                self.graph = None;
                self.settle();
            }
            Action::Letter(crate::ui::app::clicks::MOUSE) => self.switch_the_mouse(),
            Action::Letter(WATCHING) => {
                if let Some(graph) = self.graph.as_mut() {
                    graph.watch(Instant::now());
                }
                self.follow_the_graph();
            }
            Action::Move(motion) | Action::Pick(motion) | Action::Gather(motion) => {
                let drawing = pieces::drawing(self.body.get());
                let page = drawing.height as usize;
                let width = self.look.text_width(drawing.width);
                let look = self.look;
                let drawn = self.graph_pieces();
                if let Some(graph) = self.graph.as_mut() {
                    let total = pieces::height(&drawn, Acts::default(), look, width);
                    graph.scroll(motion, page, total);
                }
            }
            _ => {}
        }
    }

    pub(in crate::ui::app) fn follow_the_graph(&mut self) {
        let Some(counted) = self.graph_counted() else {
            return;
        };
        let drawn = self.graph_drawn();
        if let Some(graph) = self.graph.as_mut() {
            if let Some(drawn) = drawn {
                graph.redrawn(drawn);
            }
            graph.note(counted);
        }
    }

    pub(in crate::ui::app) fn graph_pieces(&self) -> Vec<vigil_view::Piece> {
        match &self.graph {
            None => Vec::new(),
            Some(graph) => graph.pieces(Instant::now(), self.graph_counted().flatten()),
        }
    }

    fn graph_drawn(&self) -> Option<Vec<vigil_view::Piece>> {
        let graph = self.graph.as_ref()?;
        let pane = self.pane()?;
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return None;
        };

        Some(pane.graph(snapshot, graph.row()))
    }

    fn graph_counted(&self) -> Option<Option<u64>> {
        let graph = self.graph.as_ref()?;
        let pane = self.pane()?;
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return None;
        };

        Some(pane.counted(snapshot, graph.row()))
    }
}
