#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arrangement {
    pub key: char,
    pub name: &'static str,
    pub about: Option<&'static str>,
}

impl Arrangement {
    pub fn new(key: char, name: &'static str) -> Arrangement {
        Arrangement {
            key,
            name,
            about: None,
        }
    }

    pub fn saying(self, about: &'static str) -> Arrangement {
        Arrangement {
            about: Some(about),
            ..self
        }
    }
}
