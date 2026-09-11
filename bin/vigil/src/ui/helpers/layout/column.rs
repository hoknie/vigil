pub fn fit(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let length = text.chars().count();
    if length <= width {
        let mut cell = text.to_string();
        cell.extend(std::iter::repeat_n(' ', width - length));
        return cell;
    }

    let mut cell: String = text.chars().take(width - 1).collect();
    cell.push('…');
    cell
}

pub fn right(text: &str, width: usize) -> String {
    let length = text.chars().count();
    if length >= width {
        return fit(text, width);
    }
    let mut cell = " ".repeat(width - length);
    cell.push_str(text);
    cell
}

pub fn columns(cells: &[(&str, usize)]) -> String {
    cells
        .iter()
        .map(|(text, width)| fit(text, *width))
        .collect::<Vec<_>>()
        .join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_is_exactly_as_wide_as_it_was_asked_to_be() {
        assert_eq!(fit("tcp", 6).chars().count(), 6);
        assert_eq!(fit("a-very-long-executable-name", 6).chars().count(), 6);
    }

    #[test]
    fn a_cut_cell_says_that_it_was_cut() {
        assert_eq!(fit("/usr/sbin/nginx", 10), "/usr/sbin…");
    }

    #[test]
    fn a_column_of_numbers_lines_up_on_its_right_edge_or_it_stops_being_a_column() {
        assert_eq!(right("152.0 KB", 10), "  152.0 KB");
        assert_eq!(right("7 B", 10), "       7 B");
        assert_eq!(
            right("152.0 KB", 10).chars().count(),
            right("7 B", 10).chars().count()
        );
    }

    #[test]
    fn a_number_too_wide_for_its_column_is_cut_and_says_so_like_any_other_cell() {
        assert_eq!(right("1023.9 PB", 6), "1023.…");
    }

    #[test]
    fn columns_line_up_at_eighty() {
        let row = columns(&[("tcp", 5), ("0.0.0.0:4444", 26), ("www-data", 9)]);

        assert_eq!(row.chars().count(), 5 + 2 + 26 + 2 + 9);
        assert!(row.starts_with("tcp  "));
    }
}
