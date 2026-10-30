use std::sync::Mutex;

use crate::collectors::lists::Lists;
use crate::types::Listing;

pub(super) enum Plan {
    Named {
        paths: Vec<(String, u64)>,
        beyond: Vec<String>,
    },
    Listed {
        listing: Listing,
        lists: Mutex<Lists>,
    },
}
