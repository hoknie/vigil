use ratatui::text::{Line, Span};

use crate::ui::Look;

pub const KEY: char = 't';

const LIST: &str = "t list";

const TREE: &str = "t tree";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Nesting {
    tree: bool,
}

impl Nesting {
    pub fn nested(self) -> bool {
        self.tree
    }

    pub fn toggle(&mut self) {
        self.tree = !self.tree;
    }

    pub fn about(self) -> Option<&'static str> {
        match self.tree {
            false => None,
            true => Some(
                "what each unit file says pulls it in — WantedBy, RequiredBy, PartOf, Wants, \
                 Requires — as written in the files, not what systemd has enabled",
            ),
        }
    }

    pub fn line(self, look: Look) -> Line<'static> {
        let mut spans = vec![Span::styled(" view ", look.palette.heading())];
        for (label, on) in [(LIST, !self.tree), (TREE, self.tree)] {
            spans.push(Span::styled(
                match on {
                    true => format!("[{label}]"),
                    false => format!(" {label} "),
                },
                match on {
                    true => look.palette.selected(),
                    false => look.palette.quiet(),
                },
            ));
            spans.push(Span::raw(" "));
        }
        Line::from(spans)
    }
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(nesting: Nesting, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 1));
        Paragraph::new(nesting.line(fixture::look())).render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn a_list_is_what_opens_and_the_key_puts_it_back() {
        let mut nesting = Nesting::default();
        assert!(!nesting.nested());

        nesting.toggle();
        assert!(nesting.nested());

        nesting.toggle();
        assert!(!nesting.nested(), "the same key is the way back");
    }

    #[test]
    fn the_key_that_switches_the_view_is_drawn_and_says_which_view_is_on() {
        let mut nesting = Nesting::default();

        let list = drawn(nesting, 80);
        assert!(list.contains("[t list]"), "{list}");
        assert!(list.contains("t tree"), "{list}");
        assert!(!list.contains("[t tree]"), "{list}");

        nesting.toggle();
        let tree = drawn(nesting, 80);
        assert!(tree.contains("[t tree]"), "{tree}");
        assert!(!tree.contains("[t list]"), "{tree}");
    }

    #[test]
    fn the_tree_says_it_is_drawn_from_the_files_and_not_from_what_is_enabled() {
        let mut nesting = Nesting::default();
        nesting.toggle();

        let said = nesting
            .about()
            .expect("the tree says what it is drawn from");

        assert!(said.contains("as written in the files"), "{said}");
        assert!(said.contains("not what systemd has enabled"), "{said}");
        assert_eq!(
            Nesting::default().about(),
            None,
            "the plain list keeps the sentence it already had, written in one place"
        );
    }

    #[test]
    fn it_fits_the_narrowest_terminal_this_is_read_on() {
        for width in [80u16, 120, 200] {
            let line = drawn(Nesting::default(), width);
            assert!(line.chars().count() <= width as usize, "{width}: {line}");
        }
    }
}
