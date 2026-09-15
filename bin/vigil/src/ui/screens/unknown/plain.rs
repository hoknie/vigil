use serde_json::Value;
use vigil_model::{Snapshot, class_of};
use vigil_view::{
    Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows, Showing, Sorting,
    Width, every_field, haystack, time_of_day,
};

const ROOM_FOR_THE_VALUES: u16 = 100;

const VALUES: usize = 4;

const SORTED_BY: &[&str] = &["CLASS", "OBJECT"];

const CLASSES: &str = "classes";

pub(super) struct Plain {
    reads: String,
    caption: String,
    about: String,
}

impl Plain {
    pub(super) fn of(collector: &str) -> Plain {
        Plain {
            reads: collector.to_string(),
            caption: collector.to_uppercase(),
            about: format!(
                "{collector}: a reading no screen of this build draws. Every row the agent \
                 sent is here, and every value it recorded about one of them is behind the \
                 right arrow"
            ),
        }
    }
}

impl Pane for Plain {
    fn name(&self) -> &str {
        &self.reads
    }

    fn caption(&self) -> &str {
        &self.caption
    }

    fn about(&self) -> &str {
        &self.about
    }

    fn reads(&self) -> &str {
        &self.reads
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("CLASS", Width::Fixed(14)),
            Column::new("OBJECT", Width::Share(1)),
        ];
        if room.holds(ROOM_FOR_THE_VALUES) {
            columns.push(Column::new("WHAT THE AGENT RECORDED", Width::Share(2)));
        }
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut keys: Vec<String> = reading
            .items
            .iter()
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, _)| key.clone())
            .collect();

        sort(&mut keys, showing.sorting);
        keys.into_iter().map(RowKey::of).collect()
    }

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        let mut index = Index::new(SORTED_BY.len());
        for (key, item) in &reading.items {
            index.push(
                RowKey::of(key.clone()),
                &haystack(key, item),
                0,
                (1..=SORTED_BY.len()).map(|by| sorted_on(key, by)).collect(),
            );
        }
        Some(index)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };

        let mut cells = vec![
            Cell::plain(class_of(&row.key)),
            Cell::plain(named(&row.key)),
        ];
        if room.holds(ROOM_FOR_THE_VALUES) {
            cells.push(Cell::plain(recorded(item)));
        }
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };

        every_field(class_of(&row.key), &named(&row.key), &row.key, item, &[])
    }

    fn counts(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        let mut classes: Vec<&str> = reading.items.keys().map(|key| class_of(key)).collect();
        classes.sort_unstable();
        classes.dedup();
        Some(Counts::default().counted(CLASSES, classes.len()))
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        counts: &Counts,
    ) -> String {
        footer(
            self.reads.as_str(),
            reading,
            showing,
            rows.len(),
            counts.number(CLASSES),
        )
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let mut classes: Vec<&str> = reading.items.keys().map(|key| class_of(key)).collect();
        classes.sort_unstable();
        classes.dedup();
        footer(self.reads.as_str(), reading, showing, shown, classes.len())
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        match showing.holding_back() {
            true => Notice::plain(format!("No row matches {:?}.", showing.search)).saying(
                "The search covers every value recorded about the row. Press / to change it, \
                 Esc to drop it.",
            ),
            false => Notice::plain("This reading holds no row at all."),
        }
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(
            Notice::plain("This reading holds no row at all.")
                .saying("The reading was taken and the agent found nothing to put in it."),
        )
    }

    fn nothing_was_read(&self) -> &'static str {
        "Nothing is listed here: nothing was read."
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn offers(&self) -> Offers {
        Offers::default()
    }
}

fn footer(
    reads: &str,
    reading: &Snapshot,
    showing: &Showing<'_>,
    shown: usize,
    classes: usize,
) -> String {
    let whole = reading.items.len();
    let mut parts = vec![match showing.holding_back() {
        false => format!("{shown} row(s), read at {}", time_of_day(&reading.taken_at)),
        true => format!(
            "{shown} of {whole} row(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    parts.push(format!("{classes} kind(s) of row"));
    parts.push(format!("no screen of this build draws {reads}"));
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
            showing.elsewhere
        ));
    }

    parts.join(" · ")
}

fn named(key: &str) -> String {
    match key.split_once('|') {
        Some((_, rest)) => rest.to_string(),
        None => key.to_string(),
    }
}

fn recorded(item: &Value) -> String {
    let Some(fields) = item.as_object() else {
        return said(item);
    };

    let mut parts: Vec<String> = fields
        .iter()
        .take(VALUES)
        .map(|(name, value)| format!("{}={}", name.replace('_', " "), said(value)))
        .collect();
    if fields.len() > VALUES {
        parts.push(format!("+{} more", fields.len() - VALUES));
    }

    parts.join(" · ")
}

fn said(value: &Value) -> String {
    match value {
        Value::Null => "not read".to_string(),
        Value::String(text) => text.clone(),
        Value::Bool(yes) => match yes {
            true => "yes".to_string(),
            false => "no".to_string(),
        },
        Value::Array(items) => format!("{} in all", items.len()),
        other => other.to_string(),
    }
}

fn sort(keys: &mut [String], sorting: Sorting) {
    if sorting.as_read() {
        return;
    }
    keys.sort_by(|left, right| {
        let ordering = sorted_on(left, sorting.by).cmp(&sorted_on(right, sorting.by));
        match sorting.descending {
            true => ordering.reverse(),
            false => ordering,
        }
    });
}

fn sorted_on(key: &str, by: usize) -> String {
    match by {
        1 => class_of(key).to_string(),
        2 => named(key),
        _ => String::new(),
    }
}
