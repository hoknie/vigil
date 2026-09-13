use vigil_model::Rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recorded {
    Created,
    Repeated {
        occurrences: u64,
        first_seen_at: Rfc3339,
    },
}
