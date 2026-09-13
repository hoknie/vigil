use super::pane::Pane;

pub trait Section: Send + Sync {
    fn name(&self) -> &'static str;

    fn title(&self) -> &'static str;

    fn holds(&self) -> &'static str;

    fn panes(&self) -> Vec<Box<dyn Pane>>;

    fn shows_what_has_gone(&self) -> bool {
        false
    }
}
