use crate::helpers::quoting::quoted;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Naming {
    #[default]
    Exact,
    Prefix,
    Pattern,
    Nothing,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Entry {
    pub key: String,
    pub naming: Naming,
    pub program: Option<String>,
    pub kind: Option<String>,
    pub until: Option<String>,
    pub reason: String,
}

impl Naming {
    pub fn field(self) -> Option<&'static str> {
        match self {
            Naming::Exact => Some("finding_key"),
            Naming::Prefix => Some("finding_key_prefix"),
            Naming::Pattern => Some("finding_key_match"),
            Naming::Nothing => None,
        }
    }
}

impl Entry {
    pub fn named(&self) -> &'static str {
        self.naming.field().unwrap_or("program")
    }

    pub fn object(&self) -> String {
        match (&self.program, self.naming) {
            (Some(program), Naming::Nothing) => format!("program {program}"),
            (Some(program), _) => format!("program {program} · {}", self.key),
            (None, _) => self.key.clone(),
        }
    }

    pub(crate) fn lines(&self) -> Vec<String> {
        let mut fields: Vec<(&str, &str)> = Vec::new();
        if let Some(field) = self.naming.field() {
            fields.push((field, &self.key));
        }
        if let Some(program) = &self.program {
            fields.push(("program", program));
        }
        if let Some(kind) = &self.kind {
            fields.push(("kind", kind));
        }
        if let Some(until) = &self.until {
            fields.push(("until", until));
        }
        fields.push(("reason", &self.reason));

        fields
            .into_iter()
            .enumerate()
            .map(|(at, (field, value))| {
                let lead = if at == 0 { "  - " } else { "    " };
                format!("{lead}{field}: {}", quoted(value))
            })
            .collect()
    }
}
