use crate::editor::Editor;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

pub struct Renderer<'a> {
    editor: &'a Editor<'a>,
    scroll: (u16, u16),
}

impl<'a> Renderer<'a> {
    pub fn new(editor: &'a Editor) -> Self {
        Self {
            editor,
            scroll: (0, 0),
        }
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if (self.editor.cursor.y as u16) < self.scroll.0 {
            self.scroll.0 = self.editor.cursor.y as u16;
        } else if (self.editor.cursor.y as u16) >= self.scroll.0 + area.height {
            self.scroll.0 = (self.editor.cursor.y as u16) - area.height.saturating_sub(1);
        }

        let lines = self
            .editor
            .lines
            .join("\n");

        let text_block = Paragraph::new(lines).scroll(self.scroll);

        text_block.render(area, buf);
    }
}
