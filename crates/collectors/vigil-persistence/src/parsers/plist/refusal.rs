use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlistRefusal {
    Empty,
    NotAPropertyList,
    Broken(String),
    TooDeep,
}

impl fmt::Display for PlistRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlistRefusal::Empty => write!(f, "the file is empty"),
            PlistRefusal::NotAPropertyList => {
                write!(f, "the file is neither an XML nor a binary property list")
            }
            PlistRefusal::Broken(why) => write!(f, "the property list is broken: {why}"),
            PlistRefusal::TooDeep => write!(
                f,
                "the property list nests deeper than {} levels, which no launchd job does",
                super::property_list::DEEPEST
            ),
        }
    }
}
