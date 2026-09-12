use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Os {
    pub family: String,
    pub distro: String,
    pub version: String,
    pub kernel: String,
    pub arch: String,
}
