#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupEntry {
    pub name: String,
    pub gid: u32,
    pub members: Vec<String>,
}

pub const PRIVILEGED_GROUPS: &[(&str, &str)] = &[
    ("root", "the superuser's own group"),
    ("sudo", "may run commands as any user"),
    ("wheel", "may run commands as any user"),
    ("admin", "may run commands as any user"),
    ("sudoers", "may run commands as any user"),
    (
        "docker",
        "may start a container that mounts the host filesystem: root by another route",
    ),
    (
        "lxd",
        "may start a container that mounts the host filesystem: root by another route",
    ),
    (
        "podman",
        "may start a container that mounts the host filesystem: root by another route",
    ),
    (
        "disk",
        "may write the raw block devices under the filesystem: root by another route",
    ),
    (
        "shadow",
        "may read the password database: root by another route",
    ),
];

pub fn privilege_of(group: &str) -> Option<&'static str> {
    PRIVILEGED_GROUPS
        .iter()
        .find(|(name, _)| *name == group)
        .map(|(_, why)| *why)
}

pub fn parse_group(text: &str) -> Vec<GroupEntry> {
    let mut groups = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        if fields.len() < 4 {
            continue;
        }
        let Ok(gid) = fields[2].parse::<u32>() else {
            continue;
        };

        groups.push(GroupEntry {
            name: fields[0].to_string(),
            gid,
            members: fields[3]
                .split(',')
                .map(str::trim)
                .filter(|member| !member.is_empty())
                .map(str::to_string)
                .collect(),
        });
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    const GROUP: &str = "\
root:x:0:
# a comment
sudo:x:27:deploy,alice
docker:x:998:deploy
empty:x:1001:
malformed:x:not-a-number:someone
short:line
";

    #[test]
    fn reads_the_members_and_walks_past_what_is_not_a_group() {
        let groups = parse_group(GROUP);

        assert_eq!(groups.len(), 4, "{groups:?}");
        let sudo = groups.iter().find(|g| g.name == "sudo").expect("present");
        assert_eq!(sudo.gid, 27);
        assert_eq!(sudo.members, ["deploy", "alice"]);
    }

    #[test]
    fn a_group_with_no_members_is_empty_rather_than_holding_one_blank_name() {
        let groups = parse_group(GROUP);
        let empty = groups.iter().find(|g| g.name == "empty").expect("present");

        assert!(empty.members.is_empty());
    }

    #[test]
    fn the_container_groups_are_named_as_the_root_equivalents_they_are() {
        assert!(privilege_of("docker").expect("privileged").contains("root"));
        assert!(privilege_of("sudo").is_some());
        assert_eq!(privilege_of("users"), None);
    }
}
