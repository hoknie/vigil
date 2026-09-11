use crate::ui::{Motion, Screen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Leave,
    Back,
    Refresh,
    Go(Screen),
    Sideways(isize),
    Move(Motion),
    Open,
    ToObject,
    Floor(isize),
    Search,
    Letter(char),
    Type(char),
    Erase,
    Accept,
    Abandon,
    Help,
    Ignore,
}
