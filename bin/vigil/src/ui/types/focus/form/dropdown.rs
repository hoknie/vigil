use vigil_view::Choice;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dropdown {
    at: usize,
    narrowing: String,
}

impl Dropdown {
    pub fn at(&self) -> usize {
        self.at
    }

    pub fn narrowing(&self) -> &str {
        &self.narrowing
    }

    pub fn shown(&self, choices: &[Choice]) -> Vec<usize> {
        let wanted = self.narrowing.to_lowercase();
        choices
            .iter()
            .enumerate()
            .filter(|(_, choice)| choice.name.to_lowercase().contains(&wanted))
            .map(|(at, _)| at)
            .collect()
    }

    pub fn under_the_cursor(&self, choices: &[Choice]) -> Option<usize> {
        self.shown(choices).get(self.at).copied()
    }

    pub fn step(&mut self, by: isize, choices: &[Choice]) {
        let last = self.shown(choices).len().saturating_sub(1) as isize;
        self.at = (self.at as isize + by).clamp(0, last) as usize;
    }

    pub fn narrow(&mut self, character: char) {
        self.narrowing.push(character);
        self.at = 0;
    }

    pub fn widen(&mut self) {
        self.narrowing.pop();
        self.at = 0;
    }
}
