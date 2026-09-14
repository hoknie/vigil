#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Sorting {
    pub by: usize,
    pub descending: bool,
}

pub const AS_READ: &str = "as the agent sends it";

impl Sorting {
    pub fn as_read(self) -> bool {
        self.by == 0
    }

    pub fn of(chosen: usize) -> Sorting {
        match chosen {
            0 => Sorting::default(),
            other => Sorting {
                by: (other - 1) / 2 + 1,
                descending: (other - 1) % 2 == 1,
            },
        }
    }

    pub fn chosen(self) -> usize {
        match self.by {
            0 => 0,
            by => (by - 1) * 2 + 1 + usize::from(self.descending),
        }
    }
}
