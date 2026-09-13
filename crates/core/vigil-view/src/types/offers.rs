#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offers {
    pub sorting: bool,
    pub search: bool,
    pub detail: bool,
}

impl Default for Offers {
    fn default() -> Offers {
        Offers {
            sorting: true,
            search: true,
            detail: true,
        }
    }
}

impl Offers {
    pub fn nothing() -> Offers {
        Offers {
            sorting: false,
            search: false,
            detail: false,
        }
    }

    pub fn sorted(self, sorting: bool) -> Offers {
        Offers { sorting, ..self }
    }

    pub fn searched(self, search: bool) -> Offers {
        Offers { search, ..self }
    }

    pub fn detailed(self, detail: bool) -> Offers {
        Offers { detail, ..self }
    }
}
