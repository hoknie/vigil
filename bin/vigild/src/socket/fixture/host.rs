use vigil_model::{Host, Os};

pub fn host() -> Host {
    Host {
        host_id: "1c9d8e7b4a5c6d0e".into(),
        install_id: "0199a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b".into(),
        boot_id: "boot-id".into(),
        hostname: "app-01".into(),
        fqdn: None,
        os: Os {
            family: "linux".into(),
            distro: "alpine".into(),
            version: "3.22".into(),
            kernel: "6.6.0".into(),
            arch: "x86_64".into(),
        },
        addresses: Vec::new(),
        tags: Default::default(),
        peer: None,
    }
}

pub fn mac() -> Host {
    Host {
        hostname: "studio-01".into(),
        os: Os {
            family: "macos".into(),
            distro: "macos".into(),
            version: "15.6".into(),
            kernel: "24.6.0".into(),
            arch: "aarch64".into(),
        },
        ..host()
    }
}
