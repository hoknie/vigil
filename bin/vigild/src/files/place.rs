use std::os::fd::OwnedFd;

pub struct Place {
    pub parent: OwnedFd,
    pub name: String,
}
