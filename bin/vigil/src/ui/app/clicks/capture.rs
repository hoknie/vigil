use crate::ui::Pointer;
use crate::ui::app::App;

pub const MOUSE: char = 'm';

const LET_GO: &str = "The mouse is off: the terminal selects and copies text as it does anywhere else. m turns it \
     back on.";

const TAKEN: &str = "The mouse is on: clicks reach the console. Hold Shift (Option on macOS) to \
                     select text over it, or press m.";

impl App {
    pub fn with_the_mouse(mut self, wanted: bool) -> App {
        self.pointer = Pointer::starting(wanted);
        self
    }

    pub fn wants_the_mouse(&self) -> bool {
        self.look.interactive() && self.pointer.on() && !self.copying()
    }

    pub fn resized(&self) {
        self.pointer.resized();
    }

    pub(in crate::ui::app) fn copying(&self) -> bool {
        self.paper.as_ref().is_some_and(|sheet| sheet.copyable)
    }

    pub(in crate::ui::app) fn switch_the_mouse(&mut self) {
        self.pointer.switch();
        self.message = Some(
            match self.pointer.on() {
                true => TAKEN,
                false => LET_GO,
            }
            .to_string(),
        );
    }

    #[cfg(test)]
    pub(in crate::ui::app) fn targets_under(
        &self,
        column: u16,
        row: u16,
    ) -> Vec<crate::ui::Target> {
        self.pointer.under(column, row)
    }

    #[cfg(test)]
    pub(in crate::ui::app) fn targets_drawn(&self) -> usize {
        self.pointer.recorded()
    }

    pub(in crate::ui::app) fn mouse_said(&self) -> &'static str {
        match (self.pointer.on(), self.copying()) {
            (false, _) => "mouse off",
            (true, true) => "mouse let go",
            (true, false) => "mouse on",
        }
    }
}
