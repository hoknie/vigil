#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offers {
    pub sorting: bool,
    pub search: bool,
    pub detail: bool,
    pub marking: bool,
}

impl Default for Offers {
    fn default() -> Offers {
        Offers {
            sorting: true,
            search: true,
            detail: true,
            marking: false,
        }
    }
}

impl Offers {
    pub fn nothing() -> Offers {
        Offers {
            sorting: false,
            search: false,
            detail: false,
            marking: false,
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

    pub fn marked(self, marking: bool) -> Offers {
        Offers { marking, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_offers_no_marking_until_it_says_so() {
        assert!(
            !Offers::default().marking,
            "marking rows is the first step of doing something to them, and a list that \
             grows the keys for it without asking is a list that grew a verb by accident"
        );
        assert!(Offers::default().marked(true).marking);
    }
}
