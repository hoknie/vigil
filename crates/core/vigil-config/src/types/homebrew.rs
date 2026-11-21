use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub const FORMULA: &str = "vigil";

pub const SERVICE_START: &str = "brew services start vigil";

pub const SERVICE_RESTART: &str = "brew services restart vigil";

pub const SERVICE_STATUS: &str = "brew services info vigil";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Homebrew {
    pub prefix: PathBuf,
}

impl Homebrew {
    pub fn of(program: &Path) -> Option<Homebrew> {
        let parts: Vec<Component> = program.components().collect();
        let cellar = parts
            .iter()
            .position(|part| *part == Component::Normal(OsStr::new("Cellar")))?;
        let [formula, _version, directory, _name] = parts.get(cellar + 1..)? else {
            return None;
        };
        let formula_matches = *formula == Component::Normal(OsStr::new(FORMULA));
        let directory_matches = ["bin", "sbin"]
            .iter()
            .any(|known| *directory == Component::Normal(OsStr::new(known)));
        let prefix: PathBuf = parts[..cellar].iter().collect();
        (formula_matches && directory_matches && prefix.is_absolute())
            .then_some(Homebrew { prefix })
    }

    pub fn opt_program(&self, name: &str) -> PathBuf {
        self.prefix
            .join("opt")
            .join(FORMULA)
            .join("sbin")
            .join(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_program_in_the_cellar_of_any_prefix_is_a_homebrew_installation() {
        for (program, prefix) in [
            (
                "/opt/homebrew/Cellar/vigil/0.5.0/sbin/vigild",
                "/opt/homebrew",
            ),
            ("/usr/local/Cellar/vigil/0.5.0/bin/vigil", "/usr/local"),
            (
                "/Users/ann/homebrew/Cellar/vigil/1.0.0-1/sbin/vigild",
                "/Users/ann/homebrew",
            ),
        ] {
            assert_eq!(
                Homebrew::of(Path::new(program)),
                Some(Homebrew {
                    prefix: PathBuf::from(prefix)
                }),
                "{program}"
            );
        }
    }

    #[test]
    fn a_program_outside_a_cellar_or_of_another_formula_is_not_homebrew() {
        for program in [
            "/usr/local/sbin/vigild",
            "/opt/homebrew/Cellar/other/1/bin/vigil",
            "/opt/homebrew/Cellar/vigil/1/libexec/vigil-launches-spool",
            "/opt/homebrew/Cellar/vigil/1/sbin/deeper/vigild",
            "/opt/homebrew/Cellar/vigil/sbin/vigild",
            "Cellar/vigil/1/sbin/vigild",
            "vigild",
        ] {
            assert_eq!(Homebrew::of(Path::new(program)), None, "{program}");
        }
    }

    #[test]
    fn the_service_runs_the_opt_path_that_survives_an_upgrade() {
        let brew = Homebrew::of(Path::new("/opt/homebrew/Cellar/vigil/0.5.0/sbin/vigild"))
            .expect("homebrew");

        assert_eq!(
            brew.opt_program("vigild"),
            Path::new("/opt/homebrew/opt/vigil/sbin/vigild")
        );
        assert!(
            !brew
                .opt_program("vigild")
                .to_string_lossy()
                .contains("Cellar")
        );
    }
}
