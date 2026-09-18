use crate::ui::Aim;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Aim(Aim),
    Button(usize),
    Back,
    Counting,
    Row(usize),
    Pane(usize),
    Group(usize),
    List,
    Detail,
}

impl Target {
    pub fn in_a_popup(self) -> bool {
        matches!(
            self,
            Target::Aim(Aim::Choice(_) | Aim::Option(_) | Aim::Popup)
        )
    }
}
