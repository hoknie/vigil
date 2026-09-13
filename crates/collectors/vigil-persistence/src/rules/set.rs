use super::{
    KernelModuleLoaded, NewCronJob, NewTimer, NewUnit, PreloadChanged, ShellProfileChanged,
    UnitCommandChanged,
};
use vigil_rules::RuleSet;

pub fn persistence_rules() -> RuleSet {
    RuleSet::of(vec![
        Box::new(PreloadChanged),
        Box::new(UnitCommandChanged),
        Box::new(NewUnit),
        Box::new(NewTimer),
        Box::new(NewCronJob),
        Box::new(ShellProfileChanged),
        Box::new(KernelModuleLoaded),
    ])
}
