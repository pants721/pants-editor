use crate::editor::Editor;
use ratatui::{prelude::*, widgets::*};

pub struct Renderer<'a>(&'a mut Editor);

impl<'a> Renderer<'a> {
    pub fn new(editor: &'a mut Editor) -> Self {
        Self(editor)
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if (self.0.cursor.y as u16) < self.0.scroll.0 {
            self.0.scroll.0 = self.0.cursor.y as u16;
        } else if (self.0.cursor.y as u16) >= self.0.scroll.0 + area.height {
            self.0.scroll.0 = (self.0.cursor.y as u16) - area.height.saturating_sub(1);
        }
        
        let lines = self
            .0
            .lines
            .join("\n");

        let text_block = Paragraph::new(lines).scroll(self.0.scroll);

        text_block.render(area, buf);
    }
}
