use std::sync::Mutex;

use crate::collectors::lists::Lists;
use crate::types::Listing;

pub(super) enum Plan {
    Named(Vec<(String, u64)>),
    Listed {
        listing: Listing,
        lists: Mutex<Lists>,
    },
}
