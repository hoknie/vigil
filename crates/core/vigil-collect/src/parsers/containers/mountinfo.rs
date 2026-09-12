use crate::helpers::unescaped;
use crate::parsers::holds_files_of_this_host;

const SEPARATOR: &str = "-";

const ROOT: &str = "/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountedIn {
    pub device: String,
    pub root: String,
    pub mount_point: String,
    pub filesystem: String,
}

pub fn parse_mountinfo(text: &str) -> Vec<MountedIn> {
    let mut mounted = Vec::new();

    for line in text.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let Some(at) = fields.iter().position(|field| *field == SEPARATOR) else {
            continue;
        };
        let (Some(device), Some(root), Some(mount_point), Some(filesystem)) = (
            fields.get(2),
            fields.get(3),
            fields.get(4),
            fields.get(at + 1),
        ) else {
            continue;
        };
        if root.is_empty() || mount_point.is_empty() || !device.contains(':') {
            continue;
        }

        mounted.push(MountedIn {
            device: (*device).to_string(),
            root: unescaped(root),
            mount_point: unescaped(mount_point),
            filesystem: (*filesystem).to_string(),
        });
    }

    mounted
}

pub fn paths_of_this_host(inside: &[MountedIn], host: &[MountedIn]) -> Vec<String> {
    let mut paths: Vec<String> = inside
        .iter()
        .filter(|entry| entry.mount_point != ROOT)
        .filter(|entry| holds_files_of_this_host(&entry.filesystem))
        .filter_map(|entry| known_to_this_host(entry, host))
        .collect();

    paths.sort_unstable();
    paths.dedup();
    paths
}

fn known_to_this_host(entry: &MountedIn, host: &[MountedIn]) -> Option<String> {
    let base = host
        .iter()
        .find(|mounted| mounted.device == entry.device && mounted.root == ROOT)
        .map(|mounted| mounted.mount_point.as_str())?;

    Some(joined(base, &entry.root))
}

fn joined(base: &str, root: &str) -> String {
    match (base, root) {
        (ROOT, _) => root.to_string(),
        (_, ROOT) => base.to_string(),
        _ => format!("{base}{root}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THIS_HOST: &str = "\
25 30 254:1 / / rw,relatime shared:1 - ext4 /dev/vda1 rw\n\
27 25 0:26 / /run rw,nosuid,nodev shared:5 - tmpfs tmpfs rw,size=802016k,mode=755\n\
28 25 0:21 / /proc rw,nosuid,nodev,noexec,relatime shared:6 - proc proc rw\n\
99 25 0:98 / /var/lib/docker/overlay2/ABC/merged rw,relatime shared:180 - overlay overlay rw\n";

    const FROM_A_CONTAINER: &str = "\
447 446 0:98 / / rw,relatime master:180 - overlay overlay rw\n\
448 447 0:140 / /proc rw,nosuid,nodev,noexec,relatime - proc proc rw\n\
449 447 0:141 / /dev rw,nosuid - tmpfs tmpfs rw,size=65536k,mode=755\n\
450 449 0:142 / /dev/shm rw,nosuid,nodev,noexec,relatime - tmpfs shm rw,size=65536k\n\
451 447 254:1 /var/lib/docker/containers/3ab1/resolv.conf /etc/resolv.conf rw,relatime - ext4 /dev/vda1 rw\n\
452 447 254:1 /srv/www /usr/share/nginx/html ro,relatime - ext4 /dev/vda1 ro\n\
453 447 0:26 /docker.sock /var/run/docker.sock rw,nosuid,noexec,relatime - tmpfs tmpfs rw,mode=755\n";

    const A_CONTAINER_HOLDING_THE_HOST: &str = "\
447 446 0:98 / / rw,relatime - overlay overlay rw\n\
448 447 254:1 / /host rw,relatime - ext4 /dev/vda1 rw\n";

    fn paths(inside: &str) -> Vec<String> {
        paths_of_this_host(&parse_mountinfo(inside), &parse_mountinfo(THIS_HOST))
    }

    #[test]
    fn a_path_bound_into_a_container_is_named_as_this_host_names_it_and_not_as_the_container_does()
    {
        assert_eq!(
            paths(FROM_A_CONTAINER),
            vec![
                "/run/docker.sock".to_string(),
                "/srv/www".to_string(),
                "/var/lib/docker/containers/3ab1/resolv.conf".to_string(),
            ],
            "the field a kernel writes is the path inside the filesystem it came from, so the \
             socket reads as /docker.sock until the mount table of this host says that \
             filesystem is at /run. A suppression is written about the path this host knows"
        );
    }

    #[test]
    fn a_container_that_mounted_the_whole_host_says_so_and_is_not_read_as_its_own_root() {
        assert_eq!(
            paths(A_CONTAINER_HOLDING_THE_HOST),
            vec!["/".to_string()],
            "the root of this host bound in at /host is the most dangerous mount there is, and \
             it is the one that looks most like the container's own root"
        );
    }

    #[test]
    fn a_filesystem_a_runtime_made_for_one_container_is_no_path_of_this_host() {
        let made: Vec<String> = paths(FROM_A_CONTAINER);

        for invented in ["/dev", "/dev/shm", "/proc"] {
            assert!(
                !made.iter().any(|path| path == invented),
                "a tmpfs made for the container holds nothing of this host, and reading its \
                 root as / would call every container one that mounted the host: {made:?}"
            );
        }
    }

    #[test]
    fn a_mount_this_agent_cannot_see_in_its_own_table_is_left_out_rather_than_guessed() {
        let from_a_namespace = "460 447 0:200 /secret /data rw,relatime - ext4 /dev/dm-3 rw\n";

        assert!(
            paths(from_a_namespace).is_empty(),
            "a filesystem this agent has no mount of is not a path of this host, and naming \
             the field as though it were one would put /secret in a finding about a path that \
             does not exist here"
        );
    }

    #[test]
    fn a_line_in_a_shape_we_do_not_know_is_skipped_rather_than_half_read() {
        assert!(parse_mountinfo("").is_empty());
        assert!(parse_mountinfo("447 446 0:98 / /\n").is_empty());
        assert_eq!(
            parse_mountinfo("rubbish\n447 446 254:1 /srv /srv rw - ext4 /dev/vda1 rw\n").len(),
            1
        );
    }

    #[test]
    fn the_optional_fields_a_kernel_writes_between_the_options_and_the_dash_are_read_past() {
        let with_peers =
            "453 447 254:1 /srv /srv rw,relatime shared:2 master:3 - ext4 /dev/vda1 rw\n";
        let without = "453 447 254:1 /srv /srv rw,relatime - ext4 /dev/vda1 rw\n";

        assert_eq!(parse_mountinfo(with_peers), parse_mountinfo(without));
    }
}
