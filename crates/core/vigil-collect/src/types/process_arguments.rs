#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessArguments {
    pub executable: String,
    pub arguments: Vec<String>,
}
