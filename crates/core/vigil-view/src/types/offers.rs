use vigil_model::{AccountObject, Changing, KillTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offers {
    pub sorting: bool,
    pub search: bool,
    pub detail: bool,
    pub marking: bool,
    pub killing: Option<KillTarget>,
    pub changing: Option<AccountObject>,
}

impl Default for Offers {
    fn default() -> Offers {
        Offers {
            sorting: true,
            search: true,
            detail: true,
            marking: false,
            killing: None,
            changing: None,
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
            killing: None,
            changing: None,
        }
    }

    pub fn changed(self, object: AccountObject) -> Offers {
        Offers {
            marking: self.marking || object.offers(Changing::Delete),
            changing: Some(object),
            ..self
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

    pub fn killed(self, target: KillTarget) -> Offers {
        Offers {
            marking: true,
            killing: Some(target),
            ..self
        }
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

    #[test]
    fn a_list_says_what_its_rows_are_when_it_offers_to_kill_them_and_says_nothing_otherwise() {
        assert_eq!(Offers::default().killing, None);
        assert_eq!(
            Offers::default().marked(true).killing,
            None,
            "a list that can be marked for suppressions has not thereby become a list whose \
             rows the agent is asked to stop"
        );

        let programs = Offers::default().killed(KillTarget::Program);
        assert_eq!(programs.killing, Some(KillTarget::Program));
        assert!(
            programs.marking,
            "a kill is aimed at marked rows, so a list that kills and cannot be marked is a \
             list that kills only what is under the cursor, by accident"
        );
    }

    #[test]
    fn a_list_of_accounts_says_what_its_rows_are_and_is_marked_only_where_rows_can_go() {
        assert_eq!(Offers::default().changing, None);

        let users = Offers::default().changed(AccountObject::User);
        assert_eq!(users.changing, Some(AccountObject::User));
        assert!(
            users.marking,
            "a delete is aimed at marked rows, as a kill is"
        );
        assert_eq!(
            users.killing, None,
            "and a list that changes accounts stops nothing"
        );

        let groups = Offers::default().changed(AccountObject::Group);
        assert!(groups.marking);
    }
}
