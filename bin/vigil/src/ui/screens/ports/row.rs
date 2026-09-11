use super::what::What;

pub struct Row<'a> {
    pub key: String,
    pub what: What<'a>,
}
