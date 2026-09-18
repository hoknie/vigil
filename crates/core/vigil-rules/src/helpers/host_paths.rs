const OF_THIS_HOST: &[&str] = &[
    "/", "/boot", "/dev", "/etc", "/home", "/proc", "/root", "/srv", "/sys", "/usr", "/var",
];

const A_RUNTIME_PUTS_THERE: &[&str] = &[
    "/var/lib/docker/containers/",
    "/var/lib/containers/storage/",
    "/var/lib/kubelet/pods/",
    "/var/lib/docker/volumes/",
    "/var/lib/containerd/",
];

pub fn is_a_path_of_this_host(path: &str) -> bool {
    !A_RUNTIME_PUTS_THERE.iter().any(|at| path.starts_with(at)) && under_a_watched_root(path)
}

pub fn is_a_path_of_this_host_whatever_follows(visible: &str) -> bool {
    let inside_a_root = OF_THIS_HOST
        .iter()
        .filter(|root| **root != "/")
        .any(|root| visible.starts_with(&format!("{root}/")));
    let maybe_a_runtime_directory = A_RUNTIME_PUTS_THERE
        .iter()
        .any(|at| at.starts_with(visible) || visible.starts_with(at));

    inside_a_root && !maybe_a_runtime_directory
}

fn under_a_watched_root(path: &str) -> bool {
    OF_THIS_HOST.iter().any(|root| match *root {
        "/" => path == "/",
        named => path == named || path.starts_with(&format!("{named}/")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_the_configuration_and_the_state_of_this_agent_are_paths_of_this_host() {
        for held in [
            "/",
            "/etc",
            "/etc/nginx",
            "/var/lib/vigil",
            "/proc",
            "/root/.ssh",
        ] {
            assert!(
                is_a_path_of_this_host(held),
                "{held}: what a container may write there, this host reads or runs"
            );
        }
    }

    #[test]
    fn a_directory_of_its_own_and_a_name_that_only_begins_like_a_root_are_not() {
        for own in ["/opt/site/data", "/etcetera", "/data", "relative/etc", ""] {
            assert!(
                !is_a_path_of_this_host(own),
                "{own:?} holds nothing this host runs, and a finding about it is noise"
            );
        }
    }

    #[test]
    fn the_start_of_a_path_cut_short_is_a_path_of_this_host_only_where_every_ending_would_be() {
        assert!(is_a_path_of_this_host_whatever_follows("/etc/ngi"));
        assert!(is_a_path_of_this_host_whatever_follows("/var/lib/vigil/"));
        for unsure in [
            "/et",
            "/etc",
            "/var/lib/docke",
            "/var/lib/docker/volumes/x",
            "/opt/a",
        ] {
            assert!(
                !is_a_path_of_this_host_whatever_follows(unsure),
                "{unsure:?}: an engine that cut a path short at fifteen characters left a start \
                 that may end in a directory of the runtime or in a name that only begins like \
                 a root, and a finding about what was cut off is a guess"
            );
        }
    }

    #[test]
    fn what_a_runtime_puts_under_its_own_directory_is_not_a_path_of_this_host() {
        for plumbing in [
            "/var/lib/docker/containers/3ab1/resolv.conf",
            "/var/lib/docker/volumes/site/_data",
            "/var/lib/containers/storage/volumes/etc_backup/_data",
            "/var/lib/kubelet/pods/9c1f/volumes/kube-api-access/token",
        ] {
            assert!(
                !is_a_path_of_this_host(plumbing),
                "{plumbing} is inside every container a runtime starts, and a finding about it \
                 would be a finding about every container on every host"
            );
        }
    }
}
