use vigil_model::Snapshot;
use vigil_view::{Index, Pane, Showing, Sorting, listed};

const SAMPLES: usize = 6;

const LONGEST: usize = 6;

pub(crate) fn the_index_lists_what_the_rows_list(
    pane: &dyn Pane,
    reading: &Snapshot,
    showing: Showing<'_>,
) {
    let index = pane
        .index(reading, &showing)
        .expect("a pane that builds no index is walked whole on every keystroke");
    assert_eq!(
        index.len(),
        pane.rows(reading, &showing.sorted(Sorting::default()))
            .len(),
        "{}: the index holds every row this showing lists before anything is searched for",
        pane.name()
    );

    for sorting in sortings(pane) {
        for typed in typings(&index) {
            let mut within: Option<Vec<usize>> = None;
            let mut before: Option<&String> = None;
            for search in &typed {
                let now = Showing {
                    search,
                    sorting,
                    ..showing
                };
                let (found, from_the_index) =
                    listed(pane, reading, &now, &index, within.as_deref());
                assert_eq!(
                    from_the_index,
                    pane.rows(reading, &now),
                    "{}: the search {search:?} sorted {sorting:?}, narrowed from what {:?} \
                     found, lists other rows from the index than from the reading, and the \
                     console answers every keystroke from the index",
                    pane.name(),
                    before
                );
                within = Some(found);
                before = Some(search);
            }
        }
    }
}

fn sortings(pane: &dyn Pane) -> Vec<Sorting> {
    let mut sortings = vec![Sorting::default()];
    if pane.offers().sorting {
        for by in 1..=pane.sorted_by().len() + 1 {
            for descending in [false, true] {
                sortings.push(Sorting { by, descending });
            }
        }
    }
    sortings
}

fn typings(index: &Index) -> Vec<Vec<String>> {
    let mut typings = vec![
        vec![String::new()],
        vec![String::new(), "e".to_string(), "e\u{2603}".to_string()],
        vec![String::new(), "S".to_string(), "SE".to_string()],
        vec!["zzzz nothing of the sort".to_string()],
    ];
    let step = (index.len() / SAMPLES).max(1);
    for at in (0..index.len()).step_by(step) {
        let text: Vec<char> = index.haystack(at).chars().collect();
        for from in [0, text.len() / 2, text.len().saturating_sub(LONGEST)] {
            let typed: Vec<String> = (1..=LONGEST)
                .map(|length| text.iter().skip(from).take(length).collect::<String>())
                .collect();
            let mut shouted: Vec<String> = typed.iter().map(|piece| piece.to_uppercase()).collect();
            let nothing = match typed.last() {
                Some(last) => vec![last.clone(), format!("{last}\u{2603}")],
                None => Vec::new(),
            };
            shouted.insert(0, String::new());
            typings.push(typed);
            typings.push(shouted);
            typings.push(nothing);
        }
    }
    typings
}
