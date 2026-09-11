use super::along::Along;
use crate::ui::screens::ports::Arrangement;
use crate::ui::{Program, Startup, Subject};

#[derive(Debug, Clone, Default)]
pub struct Lists {
    pub ports: Along<Arrangement>,
    pub accounts: Along<Subject>,
    pub programs: Along<Program>,
    pub startup: Along<Startup>,
}
