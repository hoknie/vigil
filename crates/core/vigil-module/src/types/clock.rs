use vigil_model::Rfc3339;

pub type Clock = fn() -> Rfc3339;
