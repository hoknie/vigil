use crate::ui::{Audience, Palette};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub palette: Palette,
    pub audience: Audience,
}

impl Look {
    pub fn new(palette: Palette, audience: Audience) -> Self {
        Look { palette, audience }
    }

    pub fn interactive(self) -> bool {
        self.audience.interactive()
    }
    pub fn text_width(self, area_width: u16) -> usize {
        match self.interactive() {
            true => area_width.saturating_sub(1) as usize,
            false => area_width as usize,
        }
    }
}
