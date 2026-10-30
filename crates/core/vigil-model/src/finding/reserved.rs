use super::kind::KnownKind;

pub const RESERVED: &[KnownKind] = &[
    KnownKind::ContainerImageUnknown,
    KnownKind::ContainerProjectPrivilegedService,
];
