#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub headline: String,
    pub detail: Vec<String>,
    pub loud: bool,
}

impl Notice {
    pub fn plain(headline: impl Into<String>) -> Notice {
        Notice {
            headline: headline.into(),
            detail: Vec::new(),
            loud: false,
        }
    }

    pub fn loud(headline: impl Into<String>) -> Notice {
        Notice {
            loud: true,
            ..Notice::plain(headline)
        }
    }

    pub fn saying(mut self, sentence: impl Into<String>) -> Notice {
        self.detail.push(sentence.into());
        self
    }
}
