use vigil_model::{AccountObject, ControlTarget, KillTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choosing {
    Sort,
    Filter,
    Kill(KillTarget),
    Control(ControlTarget),
    Delete(AccountObject),
    Unwatch,
}

impl Choosing {
    pub fn caption(self) -> &'static str {
        match self {
            Choosing::Sort => "sort by",
            Choosing::Filter => "show only",
            Choosing::Kill(KillTarget::Socket) => "close them by",
            Choosing::Kill(KillTarget::Program) => "stop them by",
            Choosing::Control(ControlTarget::Unit) => "do this to them:",
            Choosing::Control(ControlTarget::Cron) => "do this to the line:",
            Choosing::Delete(_) => "delete them:",
            Choosing::Unwatch => "stop watching it:",
        }
    }

    pub fn asks_before_acting(self) -> bool {
        matches!(
            self,
            Choosing::Kill(_) | Choosing::Control(_) | Choosing::Delete(_) | Choosing::Unwatch
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chooser {
    open: Option<Choosing>,
    at: usize,
    offered: Vec<String>,
    keys: Vec<char>,
}

pub const CANCEL: char = 'C';

impl Chooser {
    pub fn open(&mut self, what: Choosing, offered: Vec<String>, at: usize) {
        self.at = at.min(offered.len().saturating_sub(1));
        self.offered = offered;
        self.keys = Vec::new();
        self.open = Some(what);
    }

    pub fn open_by_key(&mut self, what: Choosing, offered: Vec<(char, String)>) {
        self.at = 0;
        self.keys = offered.iter().map(|(key, _)| *key).collect();
        self.offered = offered.into_iter().map(|(_, said)| said).collect();
        self.open = Some(what);
    }

    pub fn keys(&self) -> &[char] {
        &self.keys
    }

    pub fn pressed(&self, key: char) -> Option<usize> {
        self.keys.iter().position(|one| *one == key)
    }

    pub fn close(&mut self) {
        self.open = None;
        self.offered.clear();
        self.keys.clear();
        self.at = 0;
    }

    pub fn choosing(&self) -> Option<Choosing> {
        self.open
    }

    pub fn at(&self) -> usize {
        self.at
    }

    pub fn offered(&self) -> &[String] {
        &self.offered
    }

    pub fn step(&mut self, by: isize) {
        if self.offered.is_empty() {
            return;
        }
        let count = self.offered.len() as isize;
        self.at = (self.at as isize + by).rem_euclid(count) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered() -> Vec<String> {
        vec!["as read".into(), "TIME ↑".into(), "TIME ↓".into()]
    }

    #[test]
    fn a_band_opened_by_key_answers_to_the_letters_it_draws_and_to_no_other() {
        let mut chooser = Chooser::default();
        chooser.open_by_key(
            Choosing::Kill(KillTarget::Socket),
            vec![
                ('S', "stop the program".into()),
                ('K', "kill it".into()),
                ('D', "close the socket".into()),
            ],
        );

        assert_eq!(chooser.pressed('K'), Some(1));
        assert_eq!(chooser.pressed('D'), Some(2));
        assert_eq!(
            chooser.pressed('k'),
            None,
            "the letters are drawn in the case they answer to, and a band that acts on the \
             other case acts on a key the reader did not read"
        );
        assert_eq!(
            chooser.pressed(CANCEL),
            None,
            "cancel is not one of the options"
        );
    }

    #[test]
    fn a_band_opened_without_keys_offers_none_so_a_letter_falls_through_to_the_screen() {
        let mut chooser = Chooser::default();
        chooser.open(Choosing::Sort, offered(), 0);

        assert!(chooser.keys().is_empty());
        assert_eq!(chooser.pressed('T'), None);
    }

    #[test]
    fn a_console_nobody_has_pressed_a_key_on_is_choosing_nothing() {
        let chooser = Chooser::default();

        assert_eq!(chooser.choosing(), None);
        assert!(chooser.offered().is_empty());
    }

    #[test]
    fn it_opens_on_what_is_chosen_now_rather_than_at_the_top_of_the_list() {
        let mut chooser = Chooser::default();

        chooser.open(Choosing::Sort, offered(), 2);

        assert_eq!(chooser.choosing(), Some(Choosing::Sort));
        assert_eq!(chooser.at(), 2, "a reader looks for where they are, first");
    }

    #[test]
    fn the_arrows_walk_the_options_and_come_round_rather_than_stopping() {
        let mut chooser = Chooser::default();
        chooser.open(Choosing::Sort, offered(), 0);

        chooser.step(-1);
        assert_eq!(chooser.at(), 2);
        chooser.step(1);
        assert_eq!(chooser.at(), 0);
    }

    #[test]
    fn the_one_band_whose_enter_changes_this_host_is_named_and_the_others_are_not() {
        assert!(
            Choosing::Kill(KillTarget::Socket).asks_before_acting()
                && Choosing::Kill(KillTarget::Program).asks_before_acting(),
            "there is no second band after this one: choosing how is choosing to do it, so \
             this band is where the line at the foot has to say what Enter costs"
        );
        assert!(
            Choosing::Control(ControlTarget::Unit).asks_before_acting()
                && Choosing::Control(ControlTarget::Cron).asks_before_acting(),
            "the band that stops a service is the only warning before it stops: there is no \
             second band after it"
        );
        for harmless in [Choosing::Sort, Choosing::Filter] {
            assert!(
                !harmless.asks_before_acting(),
                "{harmless:?} changes nothing on the host"
            );
        }
    }

    #[test]
    fn closing_it_leaves_nothing_behind_for_the_next_screen_to_draw() {
        let mut chooser = Chooser::default();
        chooser.open(Choosing::Filter, offered(), 1);

        chooser.close();

        assert_eq!(chooser.choosing(), None);
        assert_eq!(chooser.at(), 0);
        assert!(chooser.offered().is_empty());
    }

    #[test]
    fn an_option_past_the_end_is_not_a_cursor_off_the_list() {
        let mut chooser = Chooser::default();

        chooser.open(Choosing::Sort, offered(), 99);

        assert_eq!(chooser.at(), 2);
    }
}
