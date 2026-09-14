use crate::ui::{Motion, Screen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Leave,
    Back,
    Refresh,
    Go(Screen),
    Sideways(isize),
    Move(Motion),
    Pick(Motion),
    Gather(Motion),
    Open,
    ToObject,
    Sort,
    Kill,
    Narrow,
    Search,
    Letter(char),
    Type(char),
    Erase,
    Accept,
    Abandon,
    Help,
    Ignore,
}
