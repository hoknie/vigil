use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EsloggerRefusal {
    NotJson,
    NotAnExec,
    Missing(&'static str),
}

impl fmt::Display for EsloggerRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EsloggerRefusal::NotJson => write!(f, "a line eslogger printed is not JSON"),
            EsloggerRefusal::NotAnExec => write!(f, "an event eslogger printed is not an exec"),
            EsloggerRefusal::Missing(field) => {
                write!(f, "an exec event eslogger printed carries no {field}")
            }
        }
    }
}
