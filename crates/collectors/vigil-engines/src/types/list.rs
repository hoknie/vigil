use super::engine::Engine;
use super::subject::Subject;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum List {
    Containers,
    Images,
    Volumes,
    Networks,
    Compose,
    Pods,
    Secrets,
    Registries,
}

impl List {
    pub const ALL: [List; 8] = [
        List::Containers,
        List::Images,
        List::Volumes,
        List::Networks,
        List::Compose,
        List::Pods,
        List::Secrets,
        List::Registries,
    ];

    pub fn of(engine: Engine) -> Vec<List> {
        List::ALL
            .into_iter()
            .filter(|list| {
                engine
                    .asks()
                    .iter()
                    .any(|asked| asked.subject == list.subject())
                    || matches!(list, List::Compose | List::Registries)
            })
            .collect()
    }

    pub fn name(self) -> &'static str {
        match self {
            List::Containers => "containers",
            List::Images => "images",
            List::Volumes => "volumes",
            List::Networks => "networks",
            List::Compose => "compose",
            List::Pods => "pods",
            List::Secrets => "secrets",
            List::Registries => "registries",
        }
    }

    pub fn subject(self) -> Subject {
        match self {
            List::Containers => Subject::Container,
            List::Images => Subject::Image,
            List::Volumes => Subject::Volume,
            List::Networks => Subject::Network,
            List::Compose => Subject::Project,
            List::Pods => Subject::Pod,
            List::Secrets => Subject::Secret,
            List::Registries => Subject::Registry,
        }
    }

    pub fn thing(self) -> &'static str {
        match self {
            List::Compose => "compose project",
            other => other.subject().as_str(),
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            List::Containers => "THE SELECTED CONTAINER",
            List::Images => "THE SELECTED IMAGE",
            List::Volumes => "THE SELECTED VOLUME",
            List::Networks => "THE SELECTED NETWORK",
            List::Compose => "THE SELECTED PROJECT",
            List::Pods => "THE SELECTED POD",
            List::Secrets => "THE SELECTED SECRET",
            List::Registries => "THE SELECTED REGISTRY",
        }
    }

    pub fn about(self) -> &'static str {
        match self {
            List::Containers => {
                "one row per container the engine holds, running or stopped: the image it runs, \
                 the networks it is on and what of this host it mounts"
            }
            List::Images => {
                "one row per image the engine holds: the tags it is run by, the digest it was \
                 pulled as, and how many of the engine's containers run it"
            }
            List::Volumes => {
                "one row per volume the engine holds: its driver, where it lives, and the path \
                 of this host it binds, if it binds one"
            }
            List::Networks => {
                "one row per network the engine holds: its driver, its subnets and whether a \
                 container on it can reach past it"
            }
            List::Compose => {
                "one row per compose project, read off the labels of the engine's containers: \
                 its services and the directory it was started from"
            }
            List::Pods => "one row per pod: the containers it holds and the networks they share",
            List::Secrets => {
                "one row per secret the engine keeps: its name, driver and when it was written; \
                 its value is never read"
            }
            List::Registries => {
                "one row per registry the engine is configured to pull from, read from its file \
                 under /etc, and whether it may be reached without TLS"
            }
        }
    }

    pub fn faceted(self) -> bool {
        matches!(self, List::Containers | List::Volumes | List::Networks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKER: usize = 3;

    const AROUND_A_NAME: usize = 2;

    const BETWEEN: usize = 3;

    const INSIDE_THE_FRAME: usize = 78;

    fn spread(lists: &[List]) -> usize {
        MARKER
            + lists
                .iter()
                .map(|list| list.name().chars().count() + AROUND_A_NAME)
                .sum::<usize>()
            + BETWEEN * lists.len().saturating_sub(1)
    }

    #[test]
    fn docker_lists_what_docker_has_and_podman_adds_its_pods_and_secrets() {
        assert_eq!(
            List::of(Engine::Docker)
                .into_iter()
                .map(List::name)
                .collect::<Vec<_>>(),
            vec![
                "containers",
                "images",
                "volumes",
                "networks",
                "compose",
                "registries"
            ]
        );
        assert_eq!(List::of(Engine::Podman).len(), 8);
    }

    #[test]
    fn every_list_is_named_by_one_short_word_so_the_row_of_docker_fits_eighty_columns() {
        for list in List::ALL {
            assert!(
                !list.name().contains(' ') && list.name().chars().count() <= 10,
                "{}: the second row of the menu holds every list of an engine",
                list.name()
            );
        }
        assert!(
            spread(&List::of(Engine::Docker)) <= INSIDE_THE_FRAME,
            "the six lists of docker are drawn beside each other in a terminal of eighty \
             columns; podman's eight fold into `< images >` there, as any row that does not fit"
        );
    }
}
