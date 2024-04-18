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

        let lines = self.0.lines.join("\n");

        let text_block = Paragraph::new(lines)
            .scroll(self.0.scroll)
            .style(self.0.theme().primary_style());

        text_block.render(area, buf);

        if let Some(col) = self.0.settings.color_column {
            let col_rect = Rect::new(
                col.saturating_sub(1) as u16, 
                0, 
                1, 
                (self.0.lines.len() - self.0.scroll.0 as usize).clamp(0, area.height as usize) as u16
            );
            buf.set_style(col_rect, Style::new().bg(self.0.theme().color_column));
        }
    }
}
