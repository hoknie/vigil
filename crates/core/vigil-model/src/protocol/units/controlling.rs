use serde::{Deserialize, Serialize};

use super::control_target::ControlTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Controlling {
    Stop,
    Start,
    Disable,
    Enable,
    Mask,
    Unmask,
    Comment,
    Uncomment,
}

impl Controlling {
    pub const ALL: &'static [Controlling] = &[
        Controlling::Stop,
        Controlling::Start,
        Controlling::Disable,
        Controlling::Enable,
        Controlling::Mask,
        Controlling::Unmask,
        Controlling::Comment,
        Controlling::Uncomment,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Controlling::Stop => "stop",
            Controlling::Start => "start",
            Controlling::Disable => "disable",
            Controlling::Enable => "enable",
            Controlling::Mask => "mask",
            Controlling::Unmask => "unmask",
            Controlling::Comment => "comment",
            Controlling::Uncomment => "uncomment",
        }
    }

    pub fn said(self) -> &'static str {
        match self {
            Controlling::Stop => "stop it now; the next boot starts it again",
            Controlling::Start => "start it now",
            Controlling::Disable => "leave it running; do not start it at the next boot",
            Controlling::Enable => "start it at the next boot",
            Controlling::Mask => "point it at nothing: until it is unmasked, nothing starts it",
            Controlling::Unmask => "take the mask off; what enabled it before decides again",
            Controlling::Comment => "put a # in front of the line, so cron stops running it",
            Controlling::Uncomment => "take the # off the line, so cron runs it again",
        }
    }

    pub fn target(self) -> ControlTarget {
        match self {
            Controlling::Comment | Controlling::Uncomment => ControlTarget::Cron,
            _ => ControlTarget::Unit,
        }
    }

    pub fn opposite(self) -> Controlling {
        match self {
            Controlling::Stop => Controlling::Start,
            Controlling::Start => Controlling::Stop,
            Controlling::Disable => Controlling::Enable,
            Controlling::Enable => Controlling::Disable,
            Controlling::Mask => Controlling::Unmask,
            Controlling::Unmask => Controlling::Mask,
            Controlling::Comment => Controlling::Uncomment,
            Controlling::Uncomment => Controlling::Comment,
        }
    }

    pub fn takes_something_away(self) -> bool {
        matches!(
            self,
            Controlling::Stop | Controlling::Disable | Controlling::Mask | Controlling::Comment
        )
    }
}
