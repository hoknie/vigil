use super::{ContainerDockerSocketExposed, ContainerHostMount, ContainerPrivileged};
use crate::RuleSet;

pub fn container_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(ContainerPrivileged),
        Box::new(ContainerHostMount),
        Box::new(ContainerDockerSocketExposed),
    ])
}
