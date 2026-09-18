use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::tests::harness::{
    a_configuration, drawn_at, into, number, press, typed, watching,
};
use crate::ui::fixture::screen;
use crate::ui::{Choosing, Level, Spot};

const HOSTS: &str = "file|/etc/hosts";

const BUNDLE: &str = "file|/etc/ssl/certs/ca-certificates.crt";

const ON_PATH: &str = "directory|/usr/local/bin";

fn on_the_watched_files(path: &str) -> App {
    let mut app = watching(path);
    press(&mut app, number(screen("system")));
    drawn_at(&app, 120, 40);

    for _ in 0..5 {
        if app
            .pane()
            .is_some_and(|pane| pane.name() == "watched files")
        {
            while app.level != Level::List {
                press(&mut app, KeyCode::Down);
            }
            return app;
        }
        press(&mut app, KeyCode::Right);
    }
    panic!("the files this host is configured by are drawn on no list of this section");
}

fn cursor_to(app: &mut App, key: &str) {
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == key)
        {
            return;
        }
        press(app, KeyCode::Down);
    }
    panic!("{key} is not on the list: {:?}", app.pane_keys());
}

fn held(path: &str) -> Vec<(String, Option<u64>)> {
    let text = std::fs::read_to_string(path).expect("readable");
    let named: Vec<(String, Option<u64>)> = vigil_config::paths(&text)
        .into_iter()
        .map(|watch| (watch.path, watch.ceiling_bytes))
        .collect();

    if !named.is_empty() {
        assert_eq!(
            named,
            vigil_files::watching_in(&text)
                .expect("the daemon would read it")
                .paths
                .unwrap_or_default()
                .iter()
                .map(|watched| (watched.path().to_string(), watched.ceiling_bytes()))
                .collect::<Vec<(String, Option<u64>)>>(),
            "what the file names and what the agent reads out of it are one list"
        );
    }
    named
}

fn retyped(app: &mut App, word: &str) {
    for _ in 0..12 {
        press(app, KeyCode::Backspace);
    }
    typed(app, word);
}

fn saved(app: &mut App) {
    while app.editing.as_ref().map(|editing| editing.spot()) != Some(Spot::Save) {
        press(app, KeyCode::Down);
    }
    press(app, KeyCode::Enter);
}

#[test]
fn n_on_the_watched_files_opens_a_form_that_asks_for_a_path_and_writes_it_into_the_file() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);

    press(&mut app, KeyCode::Char('n'));
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("WATCH A PATH ON THIS HOST"), "{page}");
    typed(&mut app, "/etc/sudoers");
    saved(&mut app);

    assert!(
        app.editing.is_none(),
        "the form closed on a write that went through"
    );
    assert!(
        held(&file).iter().any(|(path, _)| path == "/etc/sudoers"),
        "{:?}",
        held(&file)
    );
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("0600"), "{page}");
    assert!(
        page.contains("next round") && !page.contains("try-restart"),
        "the agent takes the watched paths up by itself, and a reader told to restart it restarts \
         a daemon for nothing, while a reader told nothing waits without knowing for how long: \
         {page}"
    );
}

#[test]
fn e_on_a_watched_file_changes_how_it_is_watched_and_leaves_the_path_where_it_was() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);
    press(&mut app, KeyCode::Char('n'));
    typed(&mut app, "/etc/sudoers");
    saved(&mut app);
    press(&mut app, KeyCode::Esc);

    cursor_to(&mut app, HOSTS);
    press(&mut app, KeyCode::Char('e'));
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("HOW /etc/hosts IS WATCHED"), "{page}");
    retyped(&mut app, "4096");
    saved(&mut app);

    assert_eq!(
        held(&file),
        vec![
            ("/etc/sudoers".to_string(), None),
            ("/etc/hosts".to_string(), Some(4096)),
        ],
        "the path this console had not been told about stays as it was, and the one that was \
         edited carries its own ceiling"
    );
}

#[test]
fn a_path_the_agent_could_never_read_back_is_refused_on_the_form_and_nothing_is_written() {
    let file = a_configuration();
    let before = std::fs::read_to_string(&file).expect("readable");
    let mut app = on_the_watched_files(&file);

    press(&mut app, KeyCode::Char('n'));
    typed(&mut app, "etc/hosts");
    saved(&mut app);

    let page = drawn_at(&app, 120, 40);
    assert!(app.editing.is_some(), "the form stays open on a refusal");
    assert!(page.contains("absolute path"), "{page}");
    assert_eq!(
        std::fs::read_to_string(&file).expect("readable"),
        before,
        "a refusal is a file nobody touched, down to the byte"
    );
    assert!(
        !std::path::Path::new(&format!("{file}.previous")).exists(),
        "a refusal writes nothing at all, so it leaves no copy of the file beside it either"
    );
}

#[test]
fn a_size_that_is_not_a_size_is_said_in_words_on_the_form() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);

    press(&mut app, KeyCode::Char('n'));
    typed(&mut app, "/etc/sudoers");
    press(&mut app, KeyCode::Down);
    typed(&mut app, "8 megabytes");
    saved(&mut app);

    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("is not a size"), "{page}");
    assert!(held(&file).is_empty(), "{:?}", held(&file));
}

#[test]
fn shift_d_asks_before_it_writes_and_takes_the_path_out_of_the_file_when_it_is_answered() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);
    press(&mut app, KeyCode::Char('n'));
    typed(&mut app, "/etc/sudoers");
    saved(&mut app);
    press(&mut app, KeyCode::Esc);
    assert_eq!(held(&file).len(), 1);

    cursor_to(&mut app, HOSTS);
    press(&mut app, KeyCode::Char('D'));
    assert_eq!(
        app.chooser.choosing(),
        Some(Choosing::Unwatch),
        "rewriting a file an operator keeps in version control is asked about first"
    );
    assert_eq!(
        held(&file).len(),
        1,
        "the band is open and nothing has been written yet"
    );

    press(&mut app, KeyCode::Char('D'));

    assert!(
        held(&file).iter().all(|(path, _)| path != "/etc/hosts"),
        "{:?}",
        held(&file)
    );
}

#[test]
fn a_directory_the_agent_watches_of_its_own_accord_is_refused_in_words_and_never_written() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);
    cursor_to(&mut app, ON_PATH);

    press(&mut app, KeyCode::Char('D'));
    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("named in no configuration file"), "{page}");
    assert_eq!(app.chooser.choosing(), None);

    press(&mut app, KeyCode::Char('e'));
    let page = drawn_at(&app, 120, 40);
    assert!(app.editing.is_none(), "{page}");
    assert!(page.contains("no configuration file"), "{page}");
}

#[test]
fn what_the_console_wrote_is_what_the_agent_reads_back_entry_by_entry() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);

    for (path, ceiling) in [("/etc/sudoers", ""), ("/etc/ssl/certs/ca.crt", "8388608")] {
        press(&mut app, KeyCode::Char('n'));
        typed(&mut app, path);
        if !ceiling.is_empty() {
            press(&mut app, KeyCode::Down);
            typed(&mut app, ceiling);
        }
        saved(&mut app);
        press(&mut app, KeyCode::Esc);
    }

    assert_eq!(
        held(&file),
        vec![
            ("/etc/sudoers".to_string(), None),
            ("/etc/ssl/certs/ca.crt".to_string(), Some(8_388_608)),
        ]
    );
}

#[test]
fn the_list_of_a_reading_nothing_writes_back_says_so_rather_than_offering_the_keys() {
    let mut app = watching(&a_configuration());
    into(&mut app, screen("system"), 120, 40);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Char('n'));

    assert!(app.editing.is_none());
    assert_eq!(app.chooser.choosing(), None);
}

#[test]
fn a_path_left_on_the_list_and_one_taken_off_it_are_both_read_back_by_the_agent() {
    let file = a_configuration();
    let mut app = on_the_watched_files(&file);
    cursor_to(&mut app, BUNDLE);

    press(&mut app, KeyCode::Char('e'));
    retyped(&mut app, "2048");
    saved(&mut app);

    assert_eq!(
        held(&file),
        vec![("/etc/ssl/certs/ca-certificates.crt".to_string(), Some(2048))]
    );
}

#[test]
fn the_keys_that_write_the_configuration_are_drawn_on_the_list_that_writes_it() {
    let app = on_the_watched_files(&a_configuration());

    let page = drawn_at(&app, 120, 40);

    for offered in ["n new", "e edit", "D delete"] {
        assert!(
            page.contains(offered),
            "a key nothing on the screen names is a key nobody presses: {offered} is missing \
             from {page}"
        );
    }
}

fn a_host_with_a_watch_list() -> (String, std::path::PathBuf) {
    let configuration = std::path::PathBuf::from(a_configuration());
    let directory = configuration.parent().expect("a directory").to_path_buf();
    let collectors = directory.join("collectors");
    let list = directory.join("watch_fs.yaml");
    std::fs::create_dir_all(&collectors).expect("a directory of collectors");
    std::fs::write(
        &configuration,
        format!(
            "state_dir: /var/lib/vigil\ncollectors_path: {}\n",
            collectors.display()
        ),
    )
    .expect("writes");
    std::fs::write(
        collectors.join("files.yaml"),
        format!(
            "files:\n  schedule: 300\n  watched_path: {}\n",
            list.display()
        ),
    )
    .expect("writes");
    std::fs::write(&list, "files:\n  - /etc/hosts\n").expect("writes");
    (configuration.display().to_string(), list)
}

#[test]
fn on_a_host_that_keeps_a_watch_list_the_form_writes_into_that_list_with_its_size() {
    let (configuration, list) = a_host_with_a_watch_list();
    let mut app = on_the_watched_files(&configuration);

    press(&mut app, KeyCode::Char('n'));
    typed(&mut app, "/etc/ssh/*.conf");
    press(&mut app, KeyCode::Down);
    typed(&mut app, "8mb");
    saved(&mut app);

    assert!(
        app.editing.is_none(),
        "the form closed on a write that went through"
    );
    let written = std::fs::read_to_string(&list).expect("readable");
    assert_eq!(
        vigil_files::watch_list_in(&written)
            .expect("the collector reads it")
            .files,
        vec![
            vigil_files::Watched::of("/etc/hosts", None),
            vigil_files::Watched::of("/etc/ssh/*.conf", Some(8 * 1024 * 1024)),
        ]
    );
    assert!(written.contains("max_file_size: 8mb"), "{written}");
}

#[test]
fn on_a_host_that_keeps_a_watch_list_shift_d_takes_the_path_out_of_that_list() {
    let (configuration, list) = a_host_with_a_watch_list();
    let mut app = on_the_watched_files(&configuration);
    cursor_to(&mut app, HOSTS);

    press(&mut app, KeyCode::Char('D'));
    press(&mut app, KeyCode::Char('D'));

    assert_eq!(
        std::fs::read_to_string(&list).expect("readable"),
        "files: []\n"
    );
}
