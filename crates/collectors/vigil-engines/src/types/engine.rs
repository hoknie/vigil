use super::asked::{Asked, CONTAINERS, IMAGES, INFORMATION, NETWORKS, PODS, SECRETS, VOLUMES};

#[cfg(not(target_os = "macos"))]
pub const DUMP_DIRECTORY: &str = "/var/lib/vigil/containers";

#[cfg(target_os = "macos")]
pub const DUMP_DIRECTORY: &str = "/usr/local/var/lib/vigil/containers";

pub const WRITTEN_BY: &str = "vigil-containers.timer";

#[cfg(not(target_os = "macos"))]
pub const WRITER: &str = "/usr/sbin/vigil-container-dump";

#[cfg(target_os = "macos")]
pub const WRITER: &str = "/usr/local/libexec/vigil/vigil-container-dump";

const DOCKER_ASKS: &[Asked] = &[IMAGES, VOLUMES, NETWORKS, CONTAINERS, INFORMATION];

const PODMAN_ASKS: &[Asked] = &[
    IMAGES,
    VOLUMES,
    NETWORKS,
    CONTAINERS,
    INFORMATION,
    PODS,
    SECRETS,
];

#[cfg(not(target_os = "macos"))]
const DOCKER_PLACES: &[&str] = &["/usr/bin/docker", "/usr/local/bin/docker", "/bin/docker"];

#[cfg(target_os = "macos")]
const DOCKER_PLACES: &[&str] = &[
    "/usr/local/bin/docker",
    "/opt/homebrew/bin/docker",
    "/Applications/Docker.app/Contents/Resources/bin/docker",
];

#[cfg(not(target_os = "macos"))]
const PODMAN_PLACES: &[&str] = &["/usr/bin/podman", "/usr/local/bin/podman", "/bin/podman"];

#[cfg(target_os = "macos")]
const PODMAN_PLACES: &[&str] = &[
    "/opt/podman/bin/podman",
    "/opt/homebrew/bin/podman",
    "/usr/local/bin/podman",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Engine {
    Docker,
    Podman,
}

impl Engine {
    pub const ALL: [Engine; 2] = [Engine::Docker, Engine::Podman];

    pub fn name(self) -> &'static str {
        match self {
            Engine::Docker => "docker",
            Engine::Podman => "podman",
        }
    }

    pub fn named(word: &str) -> Option<Engine> {
        Engine::ALL.into_iter().find(|engine| engine.name() == word)
    }

    pub fn places(self) -> &'static [&'static str] {
        match self {
            Engine::Docker => DOCKER_PLACES,
            Engine::Podman => PODMAN_PLACES,
        }
    }

    pub fn asks(self) -> &'static [Asked] {
        match self {
            Engine::Docker => DOCKER_ASKS,
            Engine::Podman => PODMAN_ASKS,
        }
    }

    pub fn dump_file(self) -> &'static str {
        match self {
            Engine::Docker => "docker.json",
            Engine::Podman => "podman.json",
        }
    }

    pub fn registries_file(self) -> &'static str {
        match self {
            Engine::Docker => "/etc/docker/daemon.json",
            Engine::Podman => "/etc/containers/registries.conf",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Subject;

    #[test]
    fn every_engine_is_looked_for_by_absolute_path_and_never_through_the_environment() {
        for engine in Engine::ALL {
            assert!(!engine.places().is_empty());
            for place in engine.places() {
                assert!(
                    place.starts_with('/') && place.ends_with(engine.name()),
                    "{place}: a bare name is looked up in PATH, and the program that runs \
                     these commands runs as root"
                );
            }
        }
    }

    #[test]
    fn podman_is_asked_the_two_things_docker_has_no_word_for_and_docker_is_not() {
        let docker: Vec<Subject> = Engine::Docker
            .asks()
            .iter()
            .map(|asked| asked.subject)
            .collect();
        let podman: Vec<Subject> = Engine::Podman
            .asks()
            .iter()
            .map(|asked| asked.subject)
            .collect();

        assert!(
            !docker.contains(&Subject::Pod) && !docker.contains(&Subject::Secret),
            "docker has no pods and no `secret ls`; asking for them is a failed command on \
             every host with docker on it, and a reader cannot tell that from a broken engine"
        );
        assert!(podman.contains(&Subject::Pod) && podman.contains(&Subject::Secret));
        for subject in docker {
            assert!(podman.contains(&subject), "{subject:?}");
        }
    }

    #[test]
    fn the_file_each_engine_is_written_to_is_named_after_it_and_after_nothing_on_the_host() {
        let mut named: Vec<&str> = Engine::ALL.into_iter().map(Engine::dump_file).collect();
        let total = named.len();
        named.sort_unstable();
        named.dedup();

        assert_eq!(
            named.len(),
            total,
            "two engines would write over each other"
        );
        for engine in Engine::ALL {
            assert_eq!(engine.dump_file(), format!("{}.json", engine.name()));
            assert_eq!(Engine::named(engine.name()), Some(engine));
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn on_a_mac_the_clients_are_looked_for_where_docker_desktop_homebrew_and_the_podman_installer_put_them()
     {
        assert_eq!(
            Engine::Docker.places(),
            &[
                "/usr/local/bin/docker",
                "/opt/homebrew/bin/docker",
                "/Applications/Docker.app/Contents/Resources/bin/docker"
            ]
        );
        assert_eq!(
            Engine::Podman.places(),
            &[
                "/opt/podman/bin/podman",
                "/opt/homebrew/bin/podman",
                "/usr/local/bin/podman"
            ]
        );
    }

    #[test]
    fn the_dump_is_written_under_the_state_directory_of_this_system_by_the_program_its_package_installs()
     {
        let installed = vigil_config::Installation::here();

        assert_eq!(
            DUMP_DIRECTORY,
            format!("{}/containers", installed.state_directory)
        );
        assert!(WRITER.ends_with("/vigil-container-dump"), "{WRITER}");
        match cfg!(target_os = "macos") {
            true => assert!(
                WRITER.starts_with("/usr/local/libexec/vigil/"),
                "the system volume of a Mac is read-only: {WRITER}"
            ),
            false => assert_eq!(WRITER, "/usr/sbin/vigil-container-dump"),
        }
    }

    #[test]
    fn the_registries_are_read_from_a_file_of_the_host_and_never_from_a_home_directory() {
        for engine in Engine::ALL {
            let path = engine.registries_file();
            assert!(
                path.starts_with("/etc/"),
                "{path}: ~/.docker/config.json holds the credentials this host pushes with, \
                 and no reading of this product opens it"
            );
        }
    }
}
