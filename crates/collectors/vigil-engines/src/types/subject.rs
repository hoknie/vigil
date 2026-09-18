#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Subject {
    Engine,
    Image,
    Volume,
    Network,
    Container,
    Pod,
    Secret,
    Project,
    Registry,
}

impl Subject {
    pub const ALL: [Subject; 9] = [
        Subject::Engine,
        Subject::Image,
        Subject::Volume,
        Subject::Network,
        Subject::Container,
        Subject::Pod,
        Subject::Secret,
        Subject::Project,
        Subject::Registry,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Subject::Engine => "engine",
            Subject::Image => "image",
            Subject::Volume => "volume",
            Subject::Network => "network",
            Subject::Container => "container",
            Subject::Pod => "pod",
            Subject::Secret => "secret",
            Subject::Project => "project",
            Subject::Registry => "registry",
        }
    }

    pub fn named(word: &str) -> Option<Subject> {
        Subject::ALL
            .into_iter()
            .find(|subject| subject.as_str() == word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_subject_is_one_word_and_reads_back_as_itself() {
        for subject in Subject::ALL {
            let word = subject.as_str();
            assert!(
                !word.contains('|'),
                "{word} carries the separator the keys of this reading are built with, so a \
                 key would split into more parts than the reader expects"
            );
            assert_eq!(Subject::named(word), Some(subject));
        }
    }

    #[test]
    fn a_word_this_build_never_heard_of_is_no_subject_rather_than_the_first_one() {
        assert_eq!(Subject::named("images"), None);
        assert_eq!(Subject::named(""), None);
    }
}
