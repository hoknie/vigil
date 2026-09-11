#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Audience {
    Person,
    Script,
}

impl Audience {
    pub fn interactive(self) -> bool {
        self == Audience::Person
    }
}
