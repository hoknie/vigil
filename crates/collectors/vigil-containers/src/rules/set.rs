use super::{ContainerDockerSocketExposed, ContainerHostMount, ContainerPrivileged};
use vigil_rules::RuleSet;

pub fn container_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(ContainerPrivileged),
        Box::new(ContainerHostMount),
        Box::new(ContainerDockerSocketExposed),
    ])
}
