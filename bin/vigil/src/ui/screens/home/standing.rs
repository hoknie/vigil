pub struct Standing {
    pub state: String,
    pub objects: Option<usize>,
    pub read: Option<String>,
    pub note: Option<String>,
    pub unwell: bool,
}

impl Standing {
    pub fn plain(state: impl Into<String>) -> Self {
        Standing {
            state: state.into(),
            objects: None,
            read: None,
            note: None,
            unwell: false,
        }
    }
}
