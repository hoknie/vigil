#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placed {
    Row(usize),
    Heading {
        first: usize,
        gathers: usize,
        opened: bool,
    },
}
