#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Room {
    columns: u16,
}

impl Room {
    pub fn of(columns: u16) -> Room {
        Room { columns }
    }

    pub fn columns(self) -> u16 {
        self.columns
    }

    pub fn holds(self, wanted: u16) -> bool {
        self.columns >= wanted
    }
}
