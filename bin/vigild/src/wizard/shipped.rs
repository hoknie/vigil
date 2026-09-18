pub struct Shipped {
    pub path: &'static str,
    pub text: &'static str,
}

pub const CONFIGURATION: &str = include_str!("../../../../config/vigil.example.yaml");

pub const WATCH_LIST: Shipped = Shipped {
    path: "watch_fs.yaml",
    text: include_str!("../../../../config/watch_fs.yaml"),
};

pub const WATCHED_BY: &str = "files";

pub const COLLECTORS: &[Shipped] = &[
    Shipped {
        path: "collectors/network.yaml",
        text: include_str!("../../../../config/collectors/network.yaml"),
    },
    Shipped {
        path: "collectors/users.yaml",
        text: include_str!("../../../../config/collectors/users.yaml"),
    },
    Shipped {
        path: "collectors/processes.yaml",
        text: include_str!("../../../../config/collectors/processes.yaml"),
    },
    Shipped {
        path: "collectors/launches.yaml",
        text: include_str!("../../../../config/collectors/launches.yaml"),
    },
    Shipped {
        path: "collectors/persistence.yaml",
        text: include_str!("../../../../config/collectors/persistence.yaml"),
    },
    Shipped {
        path: "collectors/firewall.yaml",
        text: include_str!("../../../../config/collectors/firewall.yaml"),
    },
    Shipped {
        path: "collectors/resources.yaml",
        text: include_str!("../../../../config/collectors/resources.yaml"),
    },
    Shipped {
        path: "collectors/files.yaml",
        text: include_str!("../../../../config/collectors/files.yaml"),
    },
    Shipped {
        path: "collectors/containers.yaml",
        text: include_str!("../../../../config/collectors/containers.yaml"),
    },
];
