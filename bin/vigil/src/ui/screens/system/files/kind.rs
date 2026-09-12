#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    File,
    Directory,
}

const FILE: &str = "file";

const DIRECTORY: &str = "directory";

impl Kind {
    pub fn of(key: &str) -> Option<Kind> {
        match key.split('|').next()? {
            FILE => Some(Kind::File),
            DIRECTORY => Some(Kind::Directory),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::File => FILE,
            Kind::Directory => DIRECTORY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_row_of_the_reading_is_named_by_the_word_its_key_opens_with() {
        assert_eq!(Kind::of("file|/etc/hosts"), Some(Kind::File));
        assert_eq!(Kind::of("directory|/usr/bin"), Some(Kind::Directory));
    }

    #[test]
    fn a_row_of_another_collector_is_nothing_this_screen_draws() {
        assert_eq!(Kind::of("unit|nginx.service"), None);
        assert_eq!(Kind::of("container|3ab1"), None);
    }

    #[test]
    fn a_watched_file_is_read_before_the_directory_a_program_could_be_dropped_into() {
        let mut order = vec![Kind::Directory, Kind::File];
        order.sort();

        assert_eq!(order, vec![Kind::File, Kind::Directory]);
    }
}
