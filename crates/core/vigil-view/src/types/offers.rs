use vigil_model::{AccountObject, Changing, ControlTarget, KillTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Offers {
    pub sorting: bool,
    pub search: bool,
    pub detail: bool,
    pub marking: bool,
    pub killing: Option<KillTarget>,
    pub changing: Option<AccountObject>,
    pub controlling: Option<ControlTarget>,
    pub history: bool,
    pub graph: bool,
    pub watching: bool,
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
            controlling: None,
            history: false,
            graph: false,
            watching: false,
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
            controlling: None,
            history: false,
            graph: false,
            watching: false,
        }
    }

    pub fn historied(self, history: bool) -> Offers {
        Offers { history, ..self }
    }

    pub fn graphed(self, graph: bool) -> Offers {
        Offers { graph, ..self }
    }

    pub fn watched(self, watching: bool) -> Offers {
        Offers { watching, ..self }
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

    pub fn controlled(self, target: ControlTarget) -> Offers {
        Offers {
            marking: true,
            controlling: Some(target),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_is_read_and_never_written_back_to_the_configuration_until_it_says_so() {
        assert!(
            !Offers::default().watching && !Offers::nothing().watching,
            "a list that edits the configuration file of this agent grows three keys that \
             rewrite a file an operator keeps in version control, and no list gets them by \
             accident"
        );
        assert!(Offers::default().watched(true).watching);
        assert_eq!(
            Offers::default().watched(true).changing,
            None,
            "writing a path into vigil.yaml is not the console asking the daemon to change \
             this host, and the two must not be read as one offer"
        );
    }

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
    fn a_list_offers_no_history_of_its_rows_until_it_says_so() {
        assert!(
            !Offers::default().history && !Offers::nothing().history,
            "a history is kept by the collector that reads the rows, and a key that opens an \
             empty panel on every other list is a key a reader learns to ignore"
        );
        assert!(Offers::default().historied(true).history);
        assert!(Offers::default().historied(true).sorting);
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
    fn a_list_says_what_its_rows_are_when_it_offers_to_control_them_and_says_nothing_otherwise() {
        assert_eq!(Offers::default().controlling, None);
        assert_eq!(
            Offers::default().marked(true).controlling,
            None,
            "a list that can be marked for suppressions has not thereby become a list whose \
             rows the agent is asked to stop, disable or comment out"
        );

        let units = Offers::default().controlled(ControlTarget::Unit);
        assert_eq!(units.controlling, Some(ControlTarget::Unit));
        assert!(
            units.marking,
            "control is aimed at marked rows, as a kill is, so a list that controls and \
             cannot be marked acts only on what is under the cursor, by accident"
        );
        assert_eq!(
            units.killing, None,
            "and a list that stops a unit signals no process: the two verbs are two keys and \
             two switches in vigil.yaml"
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

    #[test]
    fn a_list_offers_no_graph_of_its_rows_until_it_says_so() {
        assert!(
            !Offers::default().graph && !Offers::nothing().graph,
            "the path a packet takes is drawn by the collector that reads the interfaces, and \
             a key that opens an empty panel on every other list is a key a reader learns to \
             ignore"
        );
        assert!(Offers::default().graphed(true).graph);
        assert!(
            !Offers::default().graphed(true).marking,
            "a graph is read, and reading a row is not marking it for anything"
        );
    }
}
