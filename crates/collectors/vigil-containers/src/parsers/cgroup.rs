const IDENTIFIER_LENGTH: usize = 64;

const SHORT_LENGTH: usize = 12;

const RUNTIME_PREFIXES: &[(&str, &str)] = &[
    ("cri-containerd-", "containerd"),
    ("containerd-", "containerd"),
    ("docker-", "docker"),
    ("crio-", "cri-o"),
    ("libpod-", "podman"),
    ("conmon-", "podman"),
];

const PATH_RUNTIMES: &[(&str, &str)] = &[
    ("/docker/", "docker"),
    ("/libpod_parent/", "podman"),
    ("/kubepods", "kubernetes"),
    ("/lxc/", "lxc"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerReference {
    pub id: String,
    pub runtime: String,
}

impl ContainerReference {
    pub fn short(&self) -> &str {
        match self.id.len() > SHORT_LENGTH {
            true => &self.id[..SHORT_LENGTH],
            false => &self.id,
        }
    }
}

pub fn parse_container_reference(text: &str) -> Option<ContainerReference> {
    for line in text.lines() {
        let Some(path) = line.splitn(3, ':').nth(2) else {
            continue;
        };
        if let Some(found) = reference_in(path) {
            return Some(found);
        }
    }

    None
}

fn reference_in(path: &str) -> Option<ContainerReference> {
    let last = path.rsplit('/').next()?;
    let name = last.strip_suffix(".scope").unwrap_or(last);
    let name = name.strip_suffix(".slice").unwrap_or(name);

    for (prefix, runtime) in RUNTIME_PREFIXES {
        if let Some(rest) = name.strip_prefix(prefix)
            && is_identifier(rest)
        {
            return Some(ContainerReference {
                id: rest.to_string(),
                runtime: (*runtime).to_string(),
            });
        }
    }

    if is_identifier(name) {
        return Some(ContainerReference {
            id: name.to_string(),
            runtime: runtime_of(path).to_string(),
        });
    }

    None
}

fn runtime_of(path: &str) -> &'static str {
    for (fragment, runtime) in PATH_RUNTIMES {
        if path.contains(fragment) {
            return runtime;
        }
    }

    "unknown"
}

fn is_identifier(name: &str) -> bool {
    name.len() == IDENTIFIER_LENGTH && name.chars().all(|character| character.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5";

    fn found(text: &str) -> Option<(String, String)> {
        parse_container_reference(text)
            .map(|reference| (reference.short().to_string(), reference.runtime))
    }

    #[test]
    fn the_four_shapes_a_runtime_writes_into_a_cgroup_path_are_one_container_each() {
        assert_eq!(
            found(&format!("12:cpuset:/docker/{ID}\n")),
            Some(("3ab1c0f2d4e5".into(), "docker".into()))
        );
        assert_eq!(
            found(&format!("0::/system.slice/docker-{ID}.scope\n")),
            Some(("3ab1c0f2d4e5".into(), "docker".into()))
        );
        assert_eq!(
            found(&format!(
                "0::/kubepods/besteffort/pod9c1f/cri-containerd-{ID}.scope\n"
            )),
            Some(("3ab1c0f2d4e5".into(), "containerd".into()))
        );
        assert_eq!(
            found(&format!("0::/machine.slice/libpod-{ID}.scope\n")),
            Some(("3ab1c0f2d4e5".into(), "podman".into()))
        );
    }

    #[test]
    fn a_process_of_the_host_itself_is_in_no_container() {
        assert_eq!(found("0::/init.scope\n"), None);
        assert_eq!(found("0::/system.slice/nginx.service\n"), None);
        assert_eq!(found("0::/\n"), None);
        assert_eq!(found("12:cpuset:/\n11:memory:/user.slice\n"), None);
        assert_eq!(found(""), None);
    }

    #[test]
    fn a_runtime_this_build_has_never_heard_of_is_a_container_of_an_unknown_runtime() {
        assert_eq!(
            found(&format!("0::/some.slice/{ID}.scope\n")),
            Some(("3ab1c0f2d4e5".into(), "unknown".into())),
            "a container whose runtime is not one of the four is still a container, and \
             leaving it out would read as a host running none"
        );
    }

    #[test]
    fn a_name_that_is_not_the_shape_of_an_identifier_is_not_read_as_one() {
        assert_eq!(found("0::/docker/not-a-container\n"), None);
        assert_eq!(found(&format!("0::/docker/{}\n", &ID[..32])), None);
        assert_eq!(
            found(&format!("0::/docker/{}z\n", &ID[..63])),
            None,
            "half of an identifier taken as a whole one would be a key that never matches \
             the container again"
        );
    }

    #[test]
    fn a_line_in_a_shape_we_do_not_know_is_skipped_rather_than_half_read() {
        assert_eq!(found("rubbish\n"), None);
        assert_eq!(found("0:\n"), None);
    }
}
