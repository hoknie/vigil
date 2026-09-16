pub trait Keyed {
    fn count(&self) -> usize;

    fn key_at(&self, at: usize) -> Option<&str>;

    fn position_of(&self, key: &str) -> Option<usize>;
}

impl Keyed for [String] {
    fn count(&self) -> usize {
        self.len()
    }

    fn key_at(&self, at: usize) -> Option<&str> {
        self.get(at).map(String::as_str)
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        self.iter().position(|row| row == key)
    }
}

impl Keyed for Vec<String> {
    fn count(&self) -> usize {
        self.as_slice().count()
    }

    fn key_at(&self, at: usize) -> Option<&str> {
        self.as_slice().key_at(at)
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        self.as_slice().position_of(key)
    }
}
