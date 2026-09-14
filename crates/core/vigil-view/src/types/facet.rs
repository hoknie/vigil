#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facet {
    pub name: &'static str,
    pub value: String,
}

impl Facet {
    pub fn new(name: &'static str, value: impl Into<String>) -> Facet {
        Facet {
            name,
            value: value.into(),
        }
    }
}
