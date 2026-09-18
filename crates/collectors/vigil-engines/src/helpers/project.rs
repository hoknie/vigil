use super::labels::Labels;

pub const PROJECT: &[&str] = &[
    "com.docker.compose.project",
    "io.podman.compose.project",
    "com.docker.stack.namespace",
];

pub const SERVICE: &[&str] = &[
    "com.docker.compose.service",
    "io.podman.compose.service",
    "com.docker.swarm.service.name",
];

pub const WORKING_DIRECTORY: &[&str] = &["com.docker.compose.project.working_dir"];

pub const CONFIGURATION_FILES: &[&str] = &["com.docker.compose.project.config_files"];

pub fn of(labels: &Labels, spellings: &[&str]) -> Option<String> {
    spellings
        .iter()
        .find_map(|name| labels.get(name))
        .filter(|said| !said.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::labels::labels;
    use super::*;

    fn read(said: serde_json::Value) -> Labels {
        labels(&json!({"Labels": said}), &["Labels"])
    }

    #[test]
    fn a_container_started_by_compose_names_its_project_and_its_service_in_its_labels() {
        let read = read(json!({
            "com.docker.compose.project": "shop",
            "com.docker.compose.service": "web",
        }));

        assert_eq!(of(&read, PROJECT).as_deref(), Some("shop"));
        assert_eq!(of(&read, SERVICE).as_deref(), Some("web"));
    }

    #[test]
    fn the_same_container_under_podman_compose_names_the_same_project() {
        let read = read(json!({"io.podman.compose.project": "shop"}));

        assert_eq!(
            of(&read, PROJECT).as_deref(),
            Some("shop"),
            "the two tools write the same fact under two names, and a reader looking at one \
             host should not have to know which tool wrote its labels"
        );
    }

    #[test]
    fn a_container_nobody_started_with_compose_belongs_to_no_project_rather_than_to_an_empty_one() {
        let read = read(json!({"role": "web", "com.docker.compose.project": ""}));

        assert_eq!(of(&read, PROJECT), None);
        assert_eq!(of(&read, SERVICE), None);
    }
}
