use vigil_model::Snapshot;

use super::reader::{Reader, present};
use super::sudoers;
use crate::accounts::fields::{item, sudoers_d_sources};
use crate::accounts::step::Step;

pub fn sudo(
    reading: &Snapshot,
    who: &str,
    rules: &[String],
    read: Reader<'_>,
) -> Result<Vec<Step>, String> {
    let grant = item(reading, &format!("sudoer|{who}"))
        .ok_or_else(|| format!("the reading grants {who} no sudo"))?;
    let mut files: Vec<(&str, String)> = Vec::new();
    for path in sudoers_d_sources(grant) {
        files.push((path, present(path, None, read)?));
    }

    let Some(first) = files
        .iter()
        .position(|(_, text)| sudoers::without(text, who).1 > 0)
    else {
        return Err(format!(
            "no file in /etc/sudoers.d holds a rule for {who} any more: the reading is older \
             than the files, and nothing was changed"
        ));
    };

    Ok(files
        .iter()
        .enumerate()
        .filter_map(|(at, (path, text))| {
            let (after, found) = match at == first {
                true => sudoers::replacing(text, who, rules),
                false => sudoers::without(text, who),
            };
            if found == 0 {
                return None;
            }
            Some(match sudoers::holds_rules(&after) {
                true => Step::Sudoers {
                    path: path.to_string(),
                    text: after,
                },
                false => Step::Remove {
                    path: path.to_string(),
                    holder: None,
                },
            })
        })
        .collect())
}
