use serde_json::Value;

use super::row::field_text;

const SHORTEST_ID: usize = 12;

pub fn untagged_image(item: &Value) -> Option<&str> {
    let image = field_text(item, "image")?;
    let id = image.strip_prefix("sha256:").unwrap_or(image);

    match id.len() >= SHORTEST_ID && id.chars().all(|letter| letter.is_ascii_hexdigit()) {
        true => Some(image),
        false => None,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_container_that_names_its_image_by_an_id_runs_an_image_nothing_tags() {
        for id in ["5f0c1ad8b29e", "sha256:5f0c1ad8b29e", &"ab".repeat(32)] {
            assert!(untagged_image(&json!({"image": id})).is_some(), "{id}");
        }
        for tagged in [
            "nginx:1.27-alpine",
            "shop/api:2026.09.1",
            "cafe",
            "localhost/pause:5",
        ] {
            assert!(
                untagged_image(&json!({"image": tagged})).is_none(),
                "{tagged}: an engine prints the name a container was started by, and a name \
                 is a tag"
            );
        }
    }
}
