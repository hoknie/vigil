use std::time::Instant;

use crate::types::Said;

use super::Round;

impl Round {
    pub(super) fn read_what_the_console_asked_for(&mut self, said: &mut Said) {
        let asked = self.shared.with(|state| state.take_readings_asked_for());

        for name in asked {
            let Some(index) = self.schedule.index_of(&name) else {
                continue;
            };
            self.read(index, said);
            self.schedule.restart(index, Instant::now());
        }
    }
}
