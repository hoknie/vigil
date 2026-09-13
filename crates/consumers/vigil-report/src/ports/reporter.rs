use vigil_model::{Envelope, Finding};

use crate::{Delivery, ReportError};

pub trait Reporter: Send + Sync {
    fn name(&self) -> &str;

    fn send(&self, envelope: &Envelope) -> Result<Delivery, ReportError>;

    fn wants(&self, _finding: &Finding) -> bool {
        true
    }
}
