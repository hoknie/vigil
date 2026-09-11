#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub findings: usize,
    pub journal_bytes: u64,
}

impl Limits {
    pub fn low_water(self) -> usize {
        self.findings - self.findings / 10
    }
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            findings: 10_000,
            journal_bytes: 16 * 1024 * 1024,
        }
    }
}
