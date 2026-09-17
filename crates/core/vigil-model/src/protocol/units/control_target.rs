use serde::{Deserialize, Serialize};

use super::controlling::Controlling;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ControlTarget {
    #[default]
    Unit,
    Cron,
}

impl ControlTarget {
    pub const ALL: &'static [ControlTarget] = &[ControlTarget::Unit, ControlTarget::Cron];

    pub fn as_str(self) -> &'static str {
        match self {
            ControlTarget::Unit => "unit",
            ControlTarget::Cron => "cron",
        }
    }

    pub fn named(self) -> &'static str {
        match self {
            ControlTarget::Unit => "unit or timer",
            ControlTarget::Cron => "cron job",
        }
    }

    pub fn ways(self) -> &'static [Controlling] {
        match self {
            ControlTarget::Unit => &[
                Controlling::Stop,
                Controlling::Start,
                Controlling::Disable,
                Controlling::Enable,
                Controlling::Mask,
                Controlling::Unmask,
            ],
            ControlTarget::Cron => &[Controlling::Comment, Controlling::Uncomment],
        }
    }

    pub fn holds(self, key: &str) -> bool {
        match self {
            ControlTarget::Unit => key.starts_with("unit|") || key.starts_with("timer|"),
            ControlTarget::Cron => key.starts_with("cron|"),
        }
    }
}
