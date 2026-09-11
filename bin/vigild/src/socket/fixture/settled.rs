use serde_json::Value;
use vigil_collect::every_seconds_of_collector;
use vigil_model::{CollectorState, Settled, class_of};

use super::statuses;

pub fn settled() -> Settled {
    let mut pinned = Settled::new("status");

    for (key, answer) in statuses() {
        if class_of(&key).starts_with("collector-") {
            pinned = collector(pinned, &key, &answer);
        }
        if class_of(&key) == "agent" {
            pinned = pinned.pinning(
                key,
                "findings/capacity",
                answer["findings"]["capacity"].clone(),
                "the number of findings the daemon holds for the console to ask for",
            );
        }
    }

    pinned
}

fn collector(pinned: Settled, key: &str, answer: &Value) -> Settled {
    let name = answer["name"].as_str().unwrap_or_default().to_string();
    let pinned = pinned.pinning(
        key,
        "name",
        name.clone(),
        "the name this build gives the collector",
    );

    if answer["state"].as_str() == Some(&CollectorState::Off.to_string()) {
        return pinned;
    }

    let declared = every_seconds_of_collector(&name)
        .unwrap_or_else(|| panic!("{name} is not a collector this build ships"));

    pinned.pinning(
        key,
        "every_seconds",
        declared,
        format!("the period the collectors of this build declare for {name}"),
    )
}
