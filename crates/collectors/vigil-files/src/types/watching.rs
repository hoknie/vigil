use serde::Deserialize;

use super::watched::Watched;

pub const CEILING_BYTES: u64 = 1024 * 1024;

pub const WATCHED_BY_DEFAULT: &[&str] = &[
    "/etc/ssh/sshd_config",
    "/etc/pam.d/sshd",
    "/etc/pam.d/su",
    "/etc/nsswitch.conf",
    "/etc/login.defs",
    "/etc/hosts",
];

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Watching {
    pub paths: Vec<Watched>,
    pub ceiling_bytes: u64,
}

impl Default for Watching {
    fn default() -> Watching {
        Watching {
            paths: WATCHED_BY_DEFAULT
                .iter()
                .map(|path| Watched::Named((*path).to_string()))
                .collect(),
            ceiling_bytes: CEILING_BYTES,
        }
    }
}

impl Watching {
    pub fn check(&self) -> Result<(), String> {
        if self.ceiling_bytes == 0 {
            return Err(
                "ceiling_bytes: 0 hashes nothing, and a file nobody hashes is a file nobody \
                 watches"
                    .to_string(),
            );
        }
        for (place, watched) in self.paths.iter().enumerate() {
            watched
                .check()
                .map_err(|why| format!("paths #{}: {why}", place + 1))?;
            if self.paths[..place]
                .iter()
                .any(|before| before.path() == watched.path())
            {
                return Err(format!(
                    "paths #{}: {:?} is named twice, and the second entry watches nothing the \
                     first does not",
                    place + 1,
                    watched.path()
                ));
            }
        }

        Ok(())
    }

    pub fn hashed(&self) -> Vec<(String, u64)> {
        self.paths
            .iter()
            .map(|watched| {
                (
                    watched.path().to_string(),
                    watched.hashed_to(self.ceiling_bytes),
                )
            })
            .collect()
    }
}
