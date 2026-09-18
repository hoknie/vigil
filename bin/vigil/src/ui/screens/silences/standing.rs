#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    InForce,
    NotReadYet,
    StillHeld,
    Unasked,
}

impl Standing {
    pub fn said(self) -> &'static str {
        match self {
            Standing::InForce => "in force",
            Standing::NotReadYet => "not read yet",
            Standing::StillHeld => "still held",
            Standing::Unasked => "not known",
        }
    }

    pub fn of(written: &str, running: Option<&[String]>) -> Standing {
        match running {
            None => Standing::Unasked,
            Some(held) if held.iter().any(|one| one == written) => Standing::InForce,
            Some(_) => Standing::NotReadYet,
        }
    }
}
