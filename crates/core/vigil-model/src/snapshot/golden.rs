use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

const ASKED_TO_WRITE: &str = "VIGIL_GOLDEN";

const RECIPE: &str = "just golden";

static STAGED: AtomicUsize = AtomicUsize::new(0);

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

    pub fn reading(name: impl Into<String>) -> Golden {
        Golden {
            family: "readings",
            name: name.into(),
        }
    }

    pub fn settled(name: impl Into<String>) -> Golden {
        Golden {
            family: "settled",
            name: name.into(),
        }
    }

    fn what(&self) -> &'static str {
        match self.family {
            "readings" => "the reading of",
            "settled" => "the settled values of",
            _ => "the shape of",
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
            return write_whole(&path, document);
        }

        if self.held().as_deref() == Some(document) {
            return Ok(());
        }

        let written = self.spill(document);
        Err(format!(
            "{} {} moved and {} did not. Run `{RECIPE}` to write the sample again and read the \
             diff: that diff is what changed on the wire. What this side builds now is in {}.",
            self.what(),
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
                    "{who} does not match {} {}. The sample is {}, the agent writes it and this \
                     side rehearses on it; what this side carries is in {}. Run `{RECIPE}` only \
                     after changing a collector — otherwise the side to change is this one.",
                    self.what(),
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

fn write_whole(path: &Path, document: &str) -> Result<(), String> {
    let staged = path.with_extension(format!(
        "json.{}.{}",
        std::process::id(),
        STAGED.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&staged, document).map_err(|error| format!("{}: {error}", staged.display()))?;
    fs::rename(&staged, path).map_err(|error| {
        let _ = fs::remove_file(&staged);
        format!("{}: {error}", path.display())
    })
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom};

    use super::*;

    #[test]
    fn a_sample_written_again_replaces_the_file_whole_so_a_reader_that_opened_it_never_sees_it_emptied()
     {
        let folder = env::temp_dir().join(format!("vigil-golden-whole-{}", std::process::id()));
        fs::create_dir_all(&folder).expect("a scratch folder");
        let path = folder.join("resources.json");
        fs::write(&path, "the sample as it was\n").expect("the sample before");
        let mut reader = fs::File::open(&path).expect("a reader opens the sample");

        write_whole(&path, "the sample as it is now\n").expect("the sample is written again");

        let mut seen = String::new();
        reader
            .seek(SeekFrom::Start(0))
            .expect("the reader starts over");
        reader.read_to_string(&mut seen).expect("the reader reads");
        let now = fs::read_to_string(&path).expect("the sample after");
        let left: Vec<_> = fs::read_dir(&folder)
            .expect("the scratch folder")
            .map(|entry| entry.expect("an entry").file_name())
            .collect();
        fs::remove_dir_all(&folder).expect("the scratch folder goes");

        assert_eq!(
            seen, "the sample as it was\n",
            "the tests that write a sample and the ones that read it run side by side; a file \
             emptied before it is filled is read as nothing, and the reader fails on a sample \
             that never changed"
        );
        assert_eq!(now, "the sample as it is now\n");
        assert_eq!(
            left.len(),
            1,
            "the staged copy is renamed into place and nothing is left beside the sample: {left:?}"
        );
    }

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
    fn a_complaint_says_which_kind_of_sample_moved() {
        let shape = Golden::snapshot("no-such-collector")
            .write_or_check("{}\n")
            .expect_err("there is no sample");
        let reading = Golden::reading("no-such-collector")
            .write_or_check("{}\n")
            .expect_err("there is no sample");
        let settled = Golden::settled("no-such-answer")
            .write_or_check("{}\n")
            .expect_err("there is no sample");

        assert!(shape.contains("the shape of"), "{shape}");
        assert!(
            reading.contains("the reading of"),
            "a whole reading and its shape are two samples, and a complaint that names neither \
             sends a reader to the wrong file: {reading}"
        );
        assert!(settled.contains("the settled values of"), "{settled}");
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
