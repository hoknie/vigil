use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const ASKED_TO_WRITE: &str = "VIGIL_GOLDEN";

const RECIPE: &str = "just golden";

pub struct Golden {
    family: &'static str,
    name: String,
}

impl Golden {
    pub fn snapshot(name: impl Into<String>) -> Golden {
        Golden {
            family: "snapshots",
            name: name.into(),
        }
    }

    pub fn protocol(name: impl Into<String>) -> Golden {
        Golden {
            family: "protocol",
            name: name.into(),
        }
    }

    pub fn path(&self) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("golden")
            .join(self.family)
            .join(format!("{}.json", self.name))
    }

    pub fn held(&self) -> Option<String> {
        fs::read_to_string(self.path()).ok()
    }

    pub fn write_or_check(&self, document: &str) -> Result<(), String> {
        let path = self.path();

        if env::var(ASKED_TO_WRITE).as_deref() == Ok("write") {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("{}: {error}", parent.display()))?;
            }
            return fs::write(&path, document)
                .map_err(|error| format!("{}: {error}", path.display()));
        }

        if self.held().as_deref() == Some(document) {
            return Ok(());
        }

        let written = self.spill(document);
        Err(format!(
            "the shape of {} has moved and {} has not. Run `{RECIPE}` to write the sample again \
             and read the diff: that diff is what changed on the wire. What this side builds now \
             is in {}.",
            self.name,
            path.display(),
            written
        ))
    }

    pub fn check(&self, who: &str, document: &str) -> Result<(), String> {
        let path = self.path();

        match self.held() {
            Some(held) if held == document => Ok(()),
            Some(_) => {
                let written = self.spill(document);
                Err(format!(
                    "{who} is not the shape of {}. The sample is {}, the agent writes it and this \
                     side rehearses on it; what this side carries is in {}. Run `{RECIPE}` only \
                     after changing a collector — otherwise the side to change is this one.",
                    self.name,
                    path.display(),
                    written
                ))
            }
            None => Err(format!(
                "{who} has no sample to rehearse on: {} is missing. Run `{RECIPE}`.",
                path.display()
            )),
        }
    }

    fn spill(&self, document: &str) -> String {
        let written =
            env::temp_dir().join(format!("vigil-golden-{}-{}.json", self.family, self.name));

        match fs::write(&written, document) {
            Ok(()) => written.display().to_string(),
            Err(error) => format!(
                "nowhere: {} would not be written ({error})",
                written.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shape_that_drifted_says_which_command_writes_the_sample_again() {
        let complaint = Golden::snapshot("no-such-collector")
            .write_or_check("{}\n")
            .expect_err("there is no sample for a collector nobody wrote one for");

        assert!(complaint.contains(RECIPE), "{complaint}");
        assert!(complaint.contains("no-such-collector.json"), "{complaint}");
        assert!(complaint.contains("vigil-golden-snapshots"), "{complaint}");
    }

    #[test]
    fn a_consumer_is_told_to_change_itself_and_not_the_sample() {
        let complaint = Golden::snapshot("no-such-collector")
            .check("the console fixture", "{}\n")
            .expect_err("there is no sample");

        assert!(complaint.contains("the console fixture"), "{complaint}");
    }

    #[test]
    fn the_samples_live_beside_the_types_both_sides_link() {
        let path = Golden::protocol("status").path();

        assert!(path.ends_with("golden/protocol/status.json"), "{path:?}");
        assert!(
            path.to_string_lossy().contains("vigil-model"),
            "the crate that owns the vocabulary owns the samples: {path:?}"
        );
    }
}
