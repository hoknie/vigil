pub fn window(selected: usize, total: usize, height: usize) -> usize {
    if height == 0 || total <= height {
        return 0;
    }

    let last_possible = total - height;
    match selected >= height {
        true => (selected + 1 - height).min(last_possible),
        false => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_that_fits_never_scrolls() {
        assert_eq!(window(0, 3, 10), 0);
        assert_eq!(window(2, 3, 10), 0);
    }

    #[test]
    fn the_selected_row_stays_on_screen() {
        assert_eq!(window(0, 100, 10), 0);
        assert_eq!(window(9, 100, 10), 0);
        assert_eq!(window(10, 100, 10), 1);
        assert_eq!(
            window(99, 100, 10),
            90,
            "the end of the list is the last page"
        );
    }

    #[test]
    fn nothing_divides_by_a_terminal_with_no_room_in_it() {
        assert_eq!(window(5, 100, 0), 0);
    }
}
