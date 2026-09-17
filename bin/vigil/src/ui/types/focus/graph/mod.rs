mod monitoring;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::time::Instant;

use vigil_view::{Piece, RowKey};

use crate::ui::Motion;

pub const KEPT: usize = 60;

pub const DRAWN: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    row: RowKey,
    drawn: Vec<Piece>,
    top: usize,
    watching: bool,
    since: Option<Instant>,
    samples: VecDeque<u64>,
}

impl Graph {
    pub fn open(row: RowKey, drawn: Vec<Piece>) -> Graph {
        Graph {
            row,
            drawn,
            top: 0,
            watching: false,
            since: None,
            samples: VecDeque::new(),
        }
    }

    pub fn row(&self) -> &RowKey {
        &self.row
    }

    pub fn watching(&self) -> bool {
        self.watching
    }

    pub fn redrawn(&mut self, drawn: Vec<Piece>) {
        self.drawn = drawn;
    }

    pub fn watch(&mut self, at: Instant) {
        self.watching = !self.watching;
        self.samples.clear();
        self.since = match self.watching {
            true => Some(at),
            false => None,
        };
    }

    pub fn note(&mut self, counted: Option<u64>) {
        let Some(counted) = counted.filter(|_| self.watching) else {
            return;
        };
        if self.samples.back() == Some(&counted) {
            return;
        }
        if self.samples.len() == KEPT {
            self.samples.pop_front();
        }
        self.samples.push_back(counted);
    }

    pub fn top(&self) -> usize {
        self.top
    }

    pub fn scroll(&mut self, motion: Motion, page: usize, total: usize) {
        let last = total.saturating_sub(page);
        self.top = match (motion.distance(page), motion) {
            (Some(by), _) => self.top.saturating_add_signed(by).min(last),
            (None, Motion::First) => 0,
            (None, _) => last,
        };
    }
}
