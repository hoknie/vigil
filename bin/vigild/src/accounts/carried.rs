use vigil_model::ChangeReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Carried {
    pub report: ChangeReport,
    pub asked: Vec<String>,
}
