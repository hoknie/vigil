pub fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        let word_length = word.chars().count();
        let line_length = line.chars().count();

        if line.is_empty() {
            line.push_str(word);
        } else if line_length + 1 + word_length <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        }

        while line.chars().count() > width {
            let head: String = line.chars().take(width).collect();
            let tail: String = line.chars().skip(width).collect();
            lines.push(head);
            line = tail;
        }
    }

    if !line.is_empty() {
        lines.push(line);
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sentence_is_broken_at_spaces_and_nothing_is_lost() {
        let wrapped = wrap(
            "findings are held in memory only and are lost on a restart",
            20,
        );

        for line in &wrapped {
            assert!(line.chars().count() <= 20, "{line}");
        }
        assert_eq!(
            wrapped.join(" "),
            "findings are held in memory only and are lost on a restart"
        );
    }

    #[test]
    fn a_word_wider_than_the_line_is_cut_rather_than_dropped() {
        let wrapped = wrap("path /a/very/long/path/that/does/not/fit", 10);

        assert!(wrapped.iter().all(|line| line.chars().count() <= 10));
        assert!(wrapped.concat().contains("/a/very/lo"));
    }

    #[test]
    fn nothing_divides_by_a_terminal_with_no_room_in_it() {
        assert!(wrap("anything", 0).is_empty());
    }
}
