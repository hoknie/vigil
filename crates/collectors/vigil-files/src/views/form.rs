use vigil_model::{Changing, Snapshot};
use vigil_view::{Field, Form, RowKey};

use super::fields::text;
use crate::types::{Family, MOST_HASHED_BYTES, Watched};

pub const PATH: &str = "path";

pub const HASHED_TO: &str = "hashed to";

const ABOUT: &str = "This writes the path into files.paths in vigil.yaml on this host. Nothing \
                     on the host itself is touched, and the agent reads the list when it next \
                     starts.";

const A_DIRECTORY: &str = "the directories a program could be dropped into are watched by this \
                           agent itself and are named in no configuration file, so there is \
                           nothing here to change";

const HASHED_HINT: &str = "bytes, or empty to leave it on the ceiling the files block names; a \
                           file over its ceiling is watched by its mode and its owner alone";

const PATH_HINT: &str = "an absolute path to one file; a directory here would be a walk that \
                         hashes everything under it";

const STAYS: &str = "the path stays as it is: a path that changed is another file, watched by \
                     stopping this one and adding that one";

pub fn form(reading: &Snapshot, row: Option<&RowKey>, changing: Changing) -> Result<Form, String> {
    match changing {
        Changing::Create => Ok(fresh()),
        Changing::Update => held(reading, row),
        Changing::Delete => Err("a path is stopped from the list and not from a form".to_string()),
    }
}

pub fn path_watched(key: &str) -> Option<&str> {
    match Family::of(key)? {
        Family::File => key.split_once('|').map(|(_, path)| path),
        Family::Directory => None,
    }
}

pub fn asked_for(form: &Form) -> Result<Watched, String> {
    let path = form.text(PATH).unwrap_or_default().trim().to_string();
    let said = form.text(HASHED_TO).unwrap_or_default().trim().to_string();

    let ceiling_bytes = match said.is_empty() {
        true => None,
        false => Some(said.parse::<u64>().map_err(|_| {
            format!(
                "{HASHED_TO}: {said:?} is not a number of bytes, and a ceiling this agent \
                 cannot read is a file it would stop hashing without saying so"
            )
        })?),
    };

    let watched = Watched::of(path, ceiling_bytes);
    watched.check()?;
    Ok(watched)
}

fn fresh() -> Form {
    Form::new("WATCH A PATH ON THIS HOST")
        .saying(ABOUT)
        .with(Field::text(PATH, PATH, "").hinted(PATH_HINT))
        .with(Field::text(HASHED_TO, HASHED_TO, "").hinted(HASHED_HINT))
}

fn held(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let row = row.ok_or("no row is under the cursor to change")?;
    let path = path_watched(&row.key).ok_or_else(|| A_DIRECTORY.to_string())?;
    let item = reading
        .items
        .get(&row.key)
        .ok_or_else(|| format!("{path} is no longer in the reading"))?;

    Ok(Form::new(format!("HOW {path} IS WATCHED"))
        .saying(ABOUT)
        .saying(format!(
            "The most this agent hashes on one pass is {MOST_HASHED_BYTES} bytes."
        ))
        .with(Field::fixed(PATH, PATH, text(item, "path")).hinted(STAYS))
        .with(
            Field::text(
                HASHED_TO,
                HASHED_TO,
                item["ceiling_bytes"]
                    .as_u64()
                    .map(|bytes| bytes.to_string())
                    .unwrap_or_default(),
            )
            .hinted(HASHED_HINT),
        ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::files;

    fn opened(key: &str) -> Result<Form, String> {
        form(&files(), Some(&RowKey::of(key)), Changing::Update)
    }

    #[test]
    fn the_form_of_a_watched_file_opens_on_the_ceiling_that_file_is_hashed_to() {
        let form = opened("file|/etc/ssl/certs/ca-certificates.crt").expect("opens");

        assert_eq!(form.text(PATH), Some("/etc/ssl/certs/ca-certificates.crt"));
        assert_eq!(form.text(HASHED_TO), Some("8388608"));
        assert!(
            !form
                .field(PATH)
                .expect("the form names the path")
                .editable(),
            "a path typed over in place would stop watching one file and start watching \
             another under one keystroke, and the reading would say neither"
        );
    }

    #[test]
    fn a_directory_this_agent_watches_of_its_own_accord_is_refused_in_words_naming_why() {
        let refusal = opened("directory|/usr/local/bin").expect_err("must not open a form");

        assert!(refusal.contains("no configuration file"), "{refusal}");
        assert_eq!(path_watched("directory|/usr/local/bin"), None);
        assert_eq!(path_watched("file|/etc/hosts"), Some("/etc/hosts"));
    }

    #[test]
    fn a_row_that_left_the_reading_since_the_key_was_pressed_says_so_and_opens_nothing() {
        let refusal = opened("file|/etc/nothing-of-the-sort").expect_err("must not open a form");

        assert!(refusal.contains("no longer in the reading"), "{refusal}");
    }

    #[test]
    fn a_fresh_form_asks_for_a_path_and_leaves_the_ceiling_to_the_block_that_names_it() {
        let form = form(&files(), None, Changing::Create).expect("opens");

        assert_eq!(form.text(PATH), Some(""));
        assert_eq!(form.text(HASHED_TO), Some(""));
        assert_eq!(
            asked_for(&form).expect_err("an empty path watches nothing"),
            Watched::of("", None).check().expect_err("refused")
        );
    }

    #[test]
    fn what_the_form_asks_for_is_read_back_as_the_entry_the_file_will_hold() {
        let mut form = form(&files(), None, Changing::Create).expect("opens");
        if let Some(field) = form.field_mut(PATH) {
            field.entry = vigil_view::Entry::Text("  /etc/sudoers  ".into());
        }

        assert_eq!(
            asked_for(&form).expect("reads"),
            Watched::of("/etc/sudoers", None),
            "a path typed with a space at either end is the path, and writing the spaces into \
             the file would watch something the kernel has never heard of"
        );
    }

    #[test]
    fn a_ceiling_that_is_not_a_number_of_bytes_is_refused_before_anything_is_written() {
        let mut form = form(&files(), None, Changing::Create).expect("opens");
        if let Some(field) = form.field_mut(PATH) {
            field.entry = vigil_view::Entry::Text("/etc/sudoers".into());
        }
        if let Some(field) = form.field_mut(HASHED_TO) {
            field.entry = vigil_view::Entry::Text("8 MB".into());
        }

        let refusal = asked_for(&form).expect_err("must not be accepted");

        assert!(refusal.contains("not a number of bytes"), "{refusal}");
    }
}
