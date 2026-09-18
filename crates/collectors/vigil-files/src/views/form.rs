use vigil_model::{Changing, Snapshot};
use vigil_view::{Field, Form, RowKey};

use super::fields::what;
use crate::types::{Family, MOST_HASHED_BYTES, Size, Watched};

pub const PATH: &str = "path";

pub const MAX_FILE_SIZE: &str = "max file size";

const ABOUT: &str = "This writes the path into the watch list of this host: the list \
                     watched_path names, or files.paths in vigil.yaml where the configuration \
                     keeps it. Nothing on the host itself is touched, and the agent reads the \
                     list again at its next reading.";

const A_DIRECTORY: &str = "the directories a program could be dropped into are watched by this \
                           agent itself and are named in no configuration file, so there is \
                           nothing here to change";

const SIZE_HINT: &str = "bytes, or 512kb, 30mb; empty leaves it on the max_file_size the files \
                         block names. A file past it is watched by its mode and its owner alone";

const PATH_HINT: &str = "an absolute path: a file, a directory walked whole, or a mask such as \
                         /etc/ssh/*.conf. A list kept inside vigil.yaml takes files alone";

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
        Family::File | Family::Walk => key.split_once('|').map(|(_, path)| path),
        Family::Directory => None,
    }
}

pub fn asked_for(form: &Form) -> Result<Watched, String> {
    let path = form.text(PATH).unwrap_or_default().trim().to_string();
    let said = form
        .text(MAX_FILE_SIZE)
        .unwrap_or_default()
        .trim()
        .to_string();

    let ceiling_bytes = match said.is_empty() {
        true => None,
        false => Some(
            Size::read(&said)
                .map_err(|why| {
                    format!(
                        "{MAX_FILE_SIZE}: {why}, and a size this agent cannot read is a file it \
                         would stop hashing without saying so"
                    )
                })?
                .bytes(),
        ),
    };

    let watched = Watched::of(path, ceiling_bytes);
    watched.check_listed()?;
    Ok(watched)
}

fn fresh() -> Form {
    Form::new("WATCH A PATH ON THIS HOST")
        .saying(ABOUT)
        .with(Field::text(PATH, PATH, "").hinted(PATH_HINT))
        .with(Field::text(MAX_FILE_SIZE, MAX_FILE_SIZE, "").hinted(SIZE_HINT))
}

fn held(reading: &Snapshot, row: Option<&RowKey>) -> Result<Form, String> {
    let row = row.ok_or("no row is under the cursor to change")?;
    let path = path_watched(&row.key).ok_or_else(|| A_DIRECTORY.to_string())?;
    let item = reading
        .items
        .get(&row.key)
        .ok_or_else(|| format!("{path} is no longer in the reading"))?;
    if let Some(by) = item["found_by"].as_str() {
        return Err(format!(
            "{path} is watched because {by} is in the watch list, so it is changed there: open \
             that entry instead"
        ));
    }
    let size = match Family::of(&row.key) {
        Some(Family::Walk) => item["max_file_size"].as_u64(),
        _ => item["ceiling_bytes"].as_u64(),
    };

    Ok(Form::new(format!("HOW {path} IS WATCHED"))
        .saying(ABOUT)
        .saying(format!(
            "The most this agent hashes of one file on one pass is {}.",
            Size::shown(MOST_HASHED_BYTES)
        ))
        .with(Field::fixed(PATH, PATH, what(item)).hinted(STAYS))
        .with(
            Field::text(
                MAX_FILE_SIZE,
                MAX_FILE_SIZE,
                size.map(Size::shown).unwrap_or_default(),
            )
            .hinted(SIZE_HINT),
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
        assert_eq!(
            form.text(MAX_FILE_SIZE),
            Some("8mb"),
            "the size is shown the way an operator writes it in the watch list"
        );
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
        assert_eq!(form.text(MAX_FILE_SIZE), Some(""));
        assert_eq!(
            asked_for(&form).expect_err("an empty path watches nothing"),
            Watched::of("", None).check_listed().expect_err("refused")
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
    fn a_size_that_is_not_a_size_is_refused_before_anything_is_written() {
        let mut form = form(&files(), None, Changing::Create).expect("opens");
        if let Some(field) = form.field_mut(PATH) {
            field.entry = vigil_view::Entry::Text("/etc/sudoers".into());
        }
        if let Some(field) = form.field_mut(MAX_FILE_SIZE) {
            field.entry = vigil_view::Entry::Text("8 megabytes".into());
        }

        let refusal = asked_for(&form).expect_err("must not be accepted");

        assert!(refusal.contains("is not a size"), "{refusal}");
    }

    #[test]
    fn a_size_written_with_a_unit_is_read_back_as_the_bytes_it_names() {
        let mut form = form(&files(), None, Changing::Create).expect("opens");
        for (field, said) in [(PATH, "/etc/ssh/*.conf"), (MAX_FILE_SIZE, "8mb")] {
            if let Some(field) = form.field_mut(field) {
                field.entry = vigil_view::Entry::Text(said.into());
            }
        }

        assert_eq!(
            asked_for(&form).expect("reads"),
            Watched::of("/etc/ssh/*.conf", Some(8 * 1024 * 1024))
        );
    }

    #[test]
    fn a_path_found_by_a_walk_is_changed_through_the_entry_that_found_it() {
        let refusal = opened("file|/etc/ssh/sshd_config.d/50-cloud-init.conf")
            .expect_err("must not open a form");

        assert!(
            refusal.contains("/etc/ssh/sshd_config.d is in the watch list"),
            "{refusal}"
        );
    }

    #[test]
    fn an_entry_of_the_watch_list_opens_on_the_size_it_walks_with() {
        let form = opened("walk|/etc/ssh/ssh_config.d/*.conf").expect("opens");

        assert_eq!(form.text(PATH), Some("/etc/ssh/ssh_config.d/*.conf"));
        assert_eq!(form.text(MAX_FILE_SIZE), Some("1mb"));
        assert_eq!(
            path_watched("walk|/etc/ssh/ssh_config.d/*.conf"),
            Some("/etc/ssh/ssh_config.d/*.conf")
        );
    }
}
