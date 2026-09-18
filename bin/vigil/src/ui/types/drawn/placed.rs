use ratatui::layout::Rect;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Placed {
    pub rows: Vec<(usize, Rect)>,
    pub names: Vec<(usize, Rect)>,
    pub groups: Vec<(usize, Rect)>,
}
