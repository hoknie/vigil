use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Changing {
    Create,
    Update,
    Delete,
}

impl Changing {
    pub const ALL: &'static [Changing] = &[Changing::Create, Changing::Update, Changing::Delete];

    pub fn as_str(self) -> &'static str {
        match self {
            Changing::Create => "create",
            Changing::Update => "update",
            Changing::Delete => "delete",
        }
    }
}
