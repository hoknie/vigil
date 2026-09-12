use super::along::Along;
use super::one::One;
use crate::ui::screens::ports::Arrangement;
use crate::ui::{Program, Startup, Subject, System};

#[derive(Debug, Clone, Default)]
pub struct Lists {
    pub ports: Along<Arrangement>,
    pub accounts: Along<Subject>,
    pub programs: Along<Program>,
    pub startup: Along<Startup>,
    pub system: Along<System>,
    pub firewall: One,
    pub containers: One,
}
