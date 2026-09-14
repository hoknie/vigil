#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    pub name: String,
    pub chosen: bool,
}

impl Choice {
    pub fn of(name: impl Into<String>, chosen: bool) -> Choice {
        Choice {
            name: name.into(),
            chosen,
        }
    }
}
