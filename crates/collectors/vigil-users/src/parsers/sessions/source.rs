#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSource {
    pub name: &'static str,
    pub path: String,
    pub present: bool,
    pub read: bool,
    pub sessions: usize,
    pub reason: Option<String>,
}

impl SessionSource {
    pub fn absent(name: &'static str, path: impl Into<String>, reason: impl Into<String>) -> Self {
        SessionSource {
            name,
            path: path.into(),
            present: false,
            read: false,
            sessions: 0,
            reason: Some(reason.into()),
        }
    }

    pub fn refused(name: &'static str, path: impl Into<String>, reason: impl Into<String>) -> Self {
        SessionSource {
            name,
            path: path.into(),
            present: true,
            read: false,
            sessions: 0,
            reason: Some(reason.into()),
        }
    }

    pub fn read(name: &'static str, path: impl Into<String>, sessions: usize) -> Self {
        SessionSource {
            name,
            path: path.into(),
            present: true,
            read: true,
            sessions,
            reason: None,
        }
    }

    pub fn saying(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn answers(&self) -> bool {
        self.present && self.read && self.reason.is_none()
    }
}
