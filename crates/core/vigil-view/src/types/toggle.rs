#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toggle {
    pub name: &'static str,
    pub about: &'static str,
    pub on: bool,
}

impl Toggle {
    pub fn on(name: &'static str, about: &'static str) -> Toggle {
        Toggle {
            name,
            about,
            on: true,
        }
    }

    pub fn off(name: &'static str, about: &'static str) -> Toggle {
        Toggle {
            on: false,
            ..Toggle::on(name, about)
        }
    }
}
