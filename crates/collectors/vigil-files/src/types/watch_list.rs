use super::devices::Devices;
use super::watched::Watched;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WatchList {
    pub files: Vec<Watched>,
    pub devices: Devices,
}
