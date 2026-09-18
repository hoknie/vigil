use vigil_rules::{BatchRule, Rule, RuleSet};

use super::{
    Appeared, ContainerFromAnUntaggedImage, ContainerOnTheHostNetwork, EngineDidNotAnswer,
    HeldOfThisHost, ImageTagMoved, RegistryWithoutTls,
};
use crate::types::{Lifecycle, Report};

pub fn engine_rules(report: &Report) -> RuleSet {
    let mut over_the_tick: Vec<Box<dyn BatchRule>> = vec![Box::new(EngineDidNotAnswer)];
    if report.images {
        over_the_tick.push(Box::new(ImageTagMoved));
    }

    let mut over_one_change: Vec<Box<dyn Rule>> = vec![
        Box::new(ContainerOnTheHostNetwork),
        Box::new(HeldOfThisHost),
        Box::new(ContainerFromAnUntaggedImage),
        Box::new(RegistryWithoutTls),
    ];
    for lifecycle in Lifecycle::ALL {
        if lifecycle.reported(report) {
            over_one_change.push(Box::new(Appeared(lifecycle)));
        }
    }

    RuleSet::new(over_the_tick, over_one_change)
}
