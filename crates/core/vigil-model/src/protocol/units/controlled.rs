use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Controlled {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    pub done: bool,
    pub said: String,
}

impl Controlled {
    pub fn done(
        key: impl Into<String>,
        object: Option<String>,
        said: impl Into<String>,
    ) -> Controlled {
        Controlled {
            key: key.into(),
            object,
            done: true,
            said: said.into(),
        }
    }

    pub fn refused(key: impl Into<String>, said: impl Into<String>) -> Controlled {
        Controlled {
            key: key.into(),
            object: None,
            done: false,
            said: said.into(),
        }
    }

    pub fn about(self, object: Option<String>) -> Controlled {
        Controlled { object, ..self }
    }
}
