use crate::ui::Screen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    screen: Screen,
    key: String,
}

impl Origin {
    pub fn new(screen: Screen, key: impl Into<String>) -> Self {
        Origin {
            screen,
            key: key.into(),
        }
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_to_come_back_to_is_a_screen_and_the_row_that_was_left_on_it() {
        let from = Origin::new(Screen::Findings, "0199a1b2-c3d4-7e5f-8a9b-000000000001");

        assert_eq!(from.screen(), Screen::Findings);
        assert_eq!(from.key(), "0199a1b2-c3d4-7e5f-8a9b-000000000001");
    }
}
