use vigil_config::Suppression;

use super::standing::Standing;
use crate::ui::types::focus::silences::Silences;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub object: String,
    pub kind: String,
    pub until: String,
    pub reason: String,
    pub file: String,
    pub written: Option<usize>,
    pub standing: Standing,
}

pub fn shown(silences: &Silences, running: Option<&[String]>) -> Vec<Shown> {
    let mut shown: Vec<Shown> = silences
        .written()
        .iter()
        .enumerate()
        .map(|(at, silenced)| {
            let suppression = &silenced.suppression;
            Shown {
                object: object(suppression),
                kind: suppression
                    .kind
                    .clone()
                    .unwrap_or_else(|| "every kind".to_string()),
                until: suppression.until.as_deref().map(until).unwrap_or_default(),
                reason: suppression.reason.clone(),
                file: silenced.file.display().to_string(),
                written: Some(at),
                standing: Standing::of(&suppression.describe(), running),
            }
        })
        .collect();

    let written: Vec<String> = silences
        .written()
        .iter()
        .map(|silenced| silenced.suppression.describe())
        .collect();
    for held in running.unwrap_or_default() {
        if written.contains(held) {
            continue;
        }
        shown.push(Shown {
            object: held.clone(),
            kind: String::new(),
            until: String::new(),
            reason: String::new(),
            file: String::new(),
            written: None,
            standing: Standing::StillHeld,
        });
    }
    shown
}

fn object(suppression: &Suppression) -> String {
    match (&suppression.finding_key, &suppression.finding_key_prefix) {
        (Some(exact), _) => exact.clone(),
        (_, Some(prefix)) => format!("{prefix}*"),
        _ => "anywhere".to_string(),
    }
}

fn until(moment: &str) -> String {
    moment.replacen('T', " ", 1).chars().take(16).collect()
}
