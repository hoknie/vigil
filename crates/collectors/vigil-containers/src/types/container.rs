use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Container,
    Socket,
}

const CONTAINER: &str = "container";

const SOCKET: &str = "container-socket";

pub struct ContainerView<'a> {
    key: &'a str,
    value: &'a Value,
}

impl<'a> ContainerView<'a> {
    pub fn new(key: &'a str, value: &'a Value) -> Self {
        ContainerView { key, value }
    }

    pub fn family(&self) -> Option<Family> {
        match self.key.split('|').next()? {
            SOCKET => Some(Family::Socket),
            CONTAINER => Some(Family::Container),
            _ => None,
        }
    }

    pub fn is(&self, family: Family) -> bool {
        self.family() == Some(family)
    }

    pub fn short(&self) -> &'a str {
        self.key.split('|').nth(1).unwrap_or("?")
    }

    pub fn runtime(&self) -> &'a str {
        self.value["runtime"].as_str().unwrap_or("?")
    }

    pub fn executable(&self) -> Option<&'a str> {
        self.value["exe"].as_str()
    }

    pub fn identity(&self) -> &'a str {
        self.executable().unwrap_or_else(|| self.short())
    }

    pub fn capabilities(&self) -> Option<u64> {
        let written = self.value["capabilities_effective"].as_str()?;

        u64::from_str_radix(written, 16).ok()
    }

    pub fn host_paths(&self) -> Vec<&'a str> {
        self.value["host_paths"]
            .as_array()
            .map(|paths| paths.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default()
    }

    pub fn path(&self) -> &'a str {
        self.value["path"].as_str().unwrap_or("?")
    }

    pub fn mode(&self) -> &'a str {
        self.value["mode"].as_str().unwrap_or("?")
    }

    pub fn reachable_by_anyone(&self) -> bool {
        self.mode()
            .chars()
            .last()
            .is_some_and(|digit| digit.is_ascii_digit() && digit != '0')
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::fixture as neighbours;

    #[test]
    fn an_item_of_another_collector_belongs_to_no_family_here() {
        let socket = neighbours::of_another_collector("tcp|0.0.0.0:443");

        assert_eq!(
            ContainerView::new("tcp|0.0.0.0:443", &socket).family(),
            None
        );
        assert_eq!(ContainerView::new("fs|/var", &socket).family(), None);
    }

    #[test]
    fn a_container_and_the_socket_of_its_runtime_are_two_families() {
        let container = fixture::container("/usr/sbin/nginx", "00000000a80425fb", &["/srv/www"]);
        let socket = fixture::runtime_socket("/run/docker.sock", "0660");

        assert!(ContainerView::new("container|3ab1c0f2d4e5", &container).is(Family::Container));
        assert!(
            ContainerView::new("container-socket|/run/docker.sock", &socket).is(Family::Socket)
        );
    }

    #[test]
    fn a_container_is_known_by_what_it_runs_and_falls_back_to_the_identifier_of_the_moment() {
        let named = fixture::container("/usr/sbin/nginx", "00000000a80425fb", &[]);
        let mut unnamed = named.clone();
        unnamed["exe"] = Value::Null;

        assert_eq!(
            ContainerView::new("container|3ab1c0f2d4e5", &named).identity(),
            "/usr/sbin/nginx",
            "the identifier changes every time the container is started again, so a finding \
             keyed by it could never be suppressed by anyone"
        );
        assert_eq!(
            ContainerView::new("container|3ab1c0f2d4e5", &unnamed).identity(),
            "3ab1c0f2d4e5"
        );
    }

    #[test]
    fn a_socket_anyone_on_this_host_may_write_to_is_not_one_only_its_group_may() {
        for mode in ["0666", "0777", "0662"] {
            assert!(
                ContainerView::new(
                    "container-socket|/run/docker.sock",
                    &fixture::runtime_socket("/run/docker.sock", mode)
                )
                .reachable_by_anyone(),
                "{mode}"
            );
        }
        for mode in ["0660", "0600", "0640"] {
            assert!(
                !ContainerView::new(
                    "container-socket|/run/docker.sock",
                    &fixture::runtime_socket("/run/docker.sock", mode)
                )
                .reachable_by_anyone(),
                "{mode} is what a runtime ships with, and a finding on every host that has \
                 not been touched is the noise this product dies of"
            );
        }
    }

    #[test]
    fn a_set_of_capabilities_in_a_shape_we_do_not_know_is_not_a_container_with_none() {
        let mut unreadable = fixture::container("/usr/sbin/nginx", "00000000a80425fb", &[]);
        unreadable["capabilities_effective"] = Value::Null;

        assert_eq!(
            ContainerView::new("container|3ab1c0f2d4e5", &unreadable).capabilities(),
            None
        );
    }
}
