use std::path::{Path, PathBuf};

use vigil_config::Layout;

use super::plan::Planned;

const TEMPLATE: &str = include_str!("../../../../packaging/macos/launch-agent/vigil.vigild.plist");

pub fn planned(layout: &Layout, program: &Path, configuration: &Path) -> Option<Planned> {
    let path = layout.launch_agent.as_ref()?;
    Some(Planned {
        path: PathBuf::from(path),
        text: Some(text(program, configuration, &layout.log_directory)),
        holds: Vec::new(),
    })
}

pub fn text(program: &Path, configuration: &Path, logs: &str) -> String {
    TEMPLATE
        .replace("@VIGILD@", &escaped(&program.display().to_string()))
        .replace(
            "@CONFIGURATION@",
            &escaped(&configuration.display().to_string()),
        )
        .replace("@LOGS@", &escaped(logs))
}

pub fn this_program() -> PathBuf {
    std::env::current_exe()
        .and_then(std::fs::canonicalize)
        .unwrap_or_else(|_| PathBuf::from("vigild"))
}

fn escaped(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use vigil_config::{Account, Installation};

    use super::*;

    fn layout(home: &str) -> Layout {
        Layout::for_account(
            Installation::MACOS,
            &Account {
                uid: 501,
                home: Some(PathBuf::from(home)),
            },
        )
        .expect("a layout")
    }

    #[test]
    fn the_job_of_an_account_runs_this_program_with_the_configuration_written() {
        let ann = layout("/Users/ann");

        let job = planned(
            &ann,
            Path::new("/opt/vigil/vigild"),
            Path::new("/Users/ann/.config/vigil/vigil.yaml"),
        )
        .expect("a job");

        assert_eq!(
            job.path,
            Path::new("/Users/ann/Library/LaunchAgents/vigil.vigild.plist")
        );
        let text = job.text.expect("a text");
        assert!(
            text.contains("<string>/opt/vigil/vigild</string>"),
            "{text}"
        );
        assert!(
            text.contains("<string>/Users/ann/.config/vigil/vigil.yaml</string>"),
            "{text}"
        );
        assert!(
            text.contains("<string>/Users/ann/Library/Logs/vigil/vigild.log</string>"),
            "{text}"
        );
        assert!(!text.contains('@'), "every label is filled: {text}");
        assert!(!text.contains("UserName"), "{text}");
    }

    #[test]
    fn a_home_with_an_ampersand_is_written_as_xml_can_read_it() {
        let said = text(
            Path::new("/Users/a&b/<x>/vigild"),
            Path::new("/Users/a&b/it's \"v\".yaml"),
            "/Users/a&b/Library/Logs/vigil",
        );

        assert!(said.contains("/Users/a&amp;b/&lt;x&gt;/vigild"), "{said}");
        assert!(said.contains("it&apos;s &quot;v&quot;.yaml"), "{said}");
        assert!(!said.contains("a&b"), "{said}");
    }

    #[test]
    fn root_on_a_mac_and_linux_plan_no_job_of_an_account() {
        for installation in [Installation::MACOS, Installation::LINUX] {
            let layout = Layout::system(installation);

            assert!(planned(&layout, Path::new("/x/vigild"), Path::new("/x/v.yaml")).is_none());
        }
    }
}
