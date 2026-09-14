use serde::{Deserialize, Serialize};

use super::account_change::AccountChange;
use super::account_object::AccountObject;
use super::changing::Changing;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changed {
    pub key: String,
    pub object: AccountObject,
    pub changing: Changing,
    pub done: bool,
    pub said: String,
}

impl Changed {
    pub fn done(change: &AccountChange, said: impl Into<String>) -> Changed {
        Changed {
            key: change.key(),
            object: change.object(),
            changing: change.changing(),
            done: true,
            said: said.into(),
        }
    }

    pub fn refused(change: &AccountChange, said: impl Into<String>) -> Changed {
        Changed {
            done: false,
            ..Changed::done(change, said)
        }
    }

    pub fn keyed(self, key: impl Into<String>) -> Changed {
        Changed {
            key: key.into(),
            ..self
        }
    }
}
