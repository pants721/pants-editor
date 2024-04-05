use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::editor::Editor;

pub fn ui(f: &mut Frame, editor: &Editor) {
    f.set_cursor(editor.cursor.0 as u16, editor.cursor.1 as u16);
    let buffer_block = Paragraph::new(editor.lines.join("\n"));
    let statusline_block = Paragraph::new(format!("{}:{}", editor.cursor.0, editor.cursor.1)).right_aligned();
    let mut statusline_rect = f.size();
    statusline_rect.height = 1;
    statusline_rect.y = f.size().height - 2;
    
    f.render_widget(buffer_block, f.size());
    f.render_widget(statusline_block, statusline_rect);
}
