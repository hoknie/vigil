#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Emphasis {
    #[default]
    Plain,
    Quiet,
    Label,
    Marked,
    Alarm,
}
