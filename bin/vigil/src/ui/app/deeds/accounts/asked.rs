use vigil_model::ChangeReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Asked {
    Report(ChangeReport),
    Refused {
        message: String,
        advice: Option<String>,
    },
    Trouble(String),
}
