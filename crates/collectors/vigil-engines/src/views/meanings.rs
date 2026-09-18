use crate::types::List;

pub(super) type Meant = (&'static str, &'static str, &'static str);

const LABELS: Meant = (
    "labels",
    "labels",
    "what it was labelled with; a label whose name reads as a secret has its value hidden \
     on this host before it is written anywhere",
);

const HIDDEN: Meant = (
    "labels hidden",
    "labels_redacted",
    "yes when the value of one of its labels was hidden",
);

const PROJECT: Meant = (
    "project",
    "project",
    "the compose project whose labels it carries",
);

const CONTAINERS: &[Meant] = &[
    (
        "name",
        "name",
        "the name the engine knows it by; compose names it <project>-<service>-<n>",
    ),
    (
        "id",
        "id",
        "the engine's id of it, which changes when it is created again",
    ),
    (
        "image",
        "image",
        "the image as it was started: a tag, or an id where nothing tags that image",
    ),
    (
        "image id",
        "image_id",
        "the id of that image, where the engine prints one; docker's list does not",
    ),
    PROJECT,
    ("service", "service", "the service of that project it runs"),
    ("pod", "pod", "the podman pod whose namespaces it shares"),
    (
        "networks",
        "networks",
        "the networks it is attached to; host is the network of this host itself",
    ),
    (
        "host network",
        "host_network",
        "yes when it shares the network of this host: it listens and connects as this host does",
    ),
    (
        "ports",
        "ports",
        "what it publishes on this host, as address:port->port/protocol",
    ),
    (
        "mounts",
        "mounts",
        "what is mounted into it: a volume by name, or a path; docker prints the path on this host, podman the path inside the container",
    ),
    (
        "mounts cut short",
        "mounts_truncated",
        "yes when docker cut a path at fifteen characters and only its start is known",
    ),
    LABELS,
    HIDDEN,
];

const IMAGES: &[Meant] = &[
    (
        "id",
        "id",
        "the digest of its configuration: two rows with one id are one image",
    ),
    (
        "tags",
        "tags",
        "the names it is pulled and run by; none means only its id names it",
    ),
    (
        "untagged",
        "untagged",
        "yes when no tag names it: built here, loaded from a file, or its tag moved to a newer image",
    ),
    (
        "digest",
        "digest",
        "the digest a registry answers for when it is pulled by name; an image built here has none",
    ),
    ("size", "size", "its size as the engine prints it"),
];

const VOLUMES: &[Meant] = &[
    (
        "name",
        "name",
        "its name; an anonymous volume is named by a random id",
    ),
    (
        "driver",
        "driver",
        "what stores it: local is a directory of this host",
    ),
    (
        "mountpoint",
        "mountpoint",
        "where the engine keeps it on this host",
    ),
    (
        "binds",
        "device",
        "the path of this host a bind volume points at: a container writing into the volume writes there",
    ),
    ("scope", "scope", "local, or shared across a swarm"),
    PROJECT,
    LABELS,
    HIDDEN,
];

const NETWORKS: &[Meant] = &[
    (
        "name",
        "name",
        "its name; bridge, host and none come with docker",
    ),
    (
        "id",
        "id",
        "the engine's id of it, which changes when it is created again",
    ),
    (
        "driver",
        "driver",
        "bridge is a private network of this host, host is the network of this host itself, macvlan and ipvlan put a container on the wire beside it",
    ),
    (
        "interface",
        "interface",
        "the interface of this host that carries it, where the engine prints one",
    ),
    (
        "internal",
        "internal",
        "yes when a container on it cannot reach past it",
    ),
    ("ipv6", "ipv6", "yes when it hands out IPv6 addresses"),
    (
        "subnets",
        "subnets",
        "the addresses it hands out; docker's list does not print them",
    ),
    PROJECT,
    LABELS,
    HIDDEN,
];

const COMPOSE: &[Meant] = &[
    (
        "name",
        "name",
        "the compose project, as its containers are labelled",
    ),
    (
        "services",
        "services",
        "the services of it that have a container on this engine",
    ),
    (
        "containers",
        "containers",
        "the containers of those services",
    ),
    (
        "working directory",
        "working_directory",
        "the directory it was started from",
    ),
    (
        "configuration files",
        "configuration_files",
        "the compose files it was started from",
    ),
];

const PODS: &[Meant] = &[
    ("name", "name", "its name"),
    ("id", "id", "the engine's id of it"),
    (
        "infra id",
        "infra_id",
        "the container that holds its namespaces",
    ),
    ("cgroup", "cgroup", "the cgroup it runs under"),
    (
        "containers",
        "containers",
        "the containers in it, which share its network",
    ),
    ("networks", "networks", "the networks it is attached to"),
    LABELS,
    HIDDEN,
];

const SECRETS: &[Meant] = &[
    ("name", "name", "its name, which a container asks for it by"),
    (
        "id",
        "id",
        "the engine's id of it, which changes when it is removed and created again",
    ),
    (
        "driver",
        "driver",
        "where the engine keeps the value: file is a file of this host that root reads",
    ),
    ("created at", "created_at", "when it was first written"),
    (
        "updated at",
        "updated_at",
        "when its value was last written; a later time is a new value",
    ),
    (
        "value",
        "value_redacted",
        "never read: the list of secrets does not print it, and nothing in this agent asks for it",
    ),
    LABELS,
    HIDDEN,
];

const REGISTRIES: &[Meant] = &[
    ("host", "host", "the registry, as host or host:port"),
    (
        "insecure",
        "insecure",
        "yes when the engine may reach it over plain http or without checking its certificate: whoever answers for that name decides what this host runs",
    ),
    (
        "role",
        "role",
        "configured is named in the file, mirror is asked before the default registry, search is where a name like nginx is looked up",
    ),
    ("from", "from", "the file of this host it was read from"),
];

pub(super) fn meant(list: List) -> &'static [Meant] {
    match list {
        List::Containers => CONTAINERS,
        List::Images => IMAGES,
        List::Volumes => VOLUMES,
        List::Networks => NETWORKS,
        List::Compose => COMPOSE,
        List::Pods => PODS,
        List::Secrets => SECRETS,
        List::Registries => REGISTRIES,
    }
}
