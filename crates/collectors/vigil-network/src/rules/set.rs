use super::{
    ClosedListeningPort, ExposedListeningPort, ListenFromWritablePath, ListeningBinaryDeleted,
    ListeningPortOwnerChanged, NewListeningPort,
};
use vigil_rules::RuleSet;

pub fn listening_port_rules() -> RuleSet {
    RuleSet::new(
        vec![Box::new(ExposedListeningPort)],
        vec![
            Box::new(ListenFromWritablePath),
            Box::new(ListeningBinaryDeleted),
            Box::new(NewListeningPort),
            Box::new(ListeningPortOwnerChanged),
            Box::new(ClosedListeningPort),
        ],
    )
}
