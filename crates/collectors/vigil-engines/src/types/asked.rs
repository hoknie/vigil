use super::subject::Subject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asked {
    pub subject: Subject,
    pub arguments: &'static [&'static str],
}

pub const IMAGES: Asked = Asked {
    subject: Subject::Image,
    arguments: &["image", "ls", "--digests", "--format", "json"],
};

pub const VOLUMES: Asked = Asked {
    subject: Subject::Volume,
    arguments: &["volume", "ls", "--format", "json"],
};

pub const NETWORKS: Asked = Asked {
    subject: Subject::Network,
    arguments: &["network", "ls", "--format", "json"],
};

pub const CONTAINERS: Asked = Asked {
    subject: Subject::Container,
    arguments: &["ps", "-a", "--format", "json"],
};

pub const INFORMATION: Asked = Asked {
    subject: Subject::Engine,
    arguments: &["system", "info", "--format", "json"],
};

pub const PODS: Asked = Asked {
    subject: Subject::Pod,
    arguments: &["pod", "ls", "--format", "json"],
};

pub const SECRETS: Asked = Asked {
    subject: Subject::Secret,
    arguments: &["secret", "ls", "--format", "json"],
};

#[cfg(test)]
mod tests {
    use super::*;

    const EVERY: [Asked; 7] = [
        IMAGES,
        VOLUMES,
        NETWORKS,
        CONTAINERS,
        INFORMATION,
        PODS,
        SECRETS,
    ];

    const LISTINGS: [&str; 3] = ["ls", "info", "ps"];

    const ACTS_ON_THE_HOST: [&str; 10] = [
        "rm", "rmi", "prune", "stop", "start", "kill", "exec", "run", "pull", "push",
    ];

    #[test]
    fn every_command_asks_a_list_of_what_is_there_and_never_removes_or_changes_anything() {
        for asked in EVERY {
            let words: Vec<&str> = asked.arguments.to_vec();
            assert!(
                words.iter().any(|word| LISTINGS.contains(word)),
                "{words:?}: this program lists what an engine holds, and `ls`, `ps` and \
                 `info` are the whole of what it may ask for. A word that is not one of them \
                 is this agent acting on a host it was installed to watch"
            );
            for word in words {
                assert!(
                    !ACTS_ON_THE_HOST.contains(&word),
                    "{word} is in the command line of a program this package installs and \
                     systemd runs as root"
                );
            }
        }
    }

    #[test]
    fn every_command_asks_for_json_because_a_table_is_a_format_that_changes_with_the_terminal() {
        for asked in EVERY {
            let tail = &asked.arguments[asked.arguments.len() - 2..];
            assert_eq!(
                tail,
                ["--format", "json"],
                "{:?} does not end in --format json",
                asked.arguments
            );
        }
    }

    #[test]
    fn no_argument_of_any_command_is_read_off_the_host() {
        for asked in EVERY {
            for word in asked.arguments {
                assert!(
                    word.is_ascii() && !word.contains(' '),
                    "{word:?} is not one fixed word: every argument here is written down in \
                     this file and none of them comes from the host"
                );
            }
        }
    }
}
