use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::editor::Editor;

pub fn ui(f: &mut Frame, editor: &mut Editor) {
    let mut buffer_rect = f.size();
    buffer_rect.height = f.size().height - 2;
    
    if (editor.cursor.1 as u16) < editor.scroll.0 {
        editor.scroll.0 = editor.cursor.1 as u16;
    } else if (editor.cursor.1 as u16) >= editor.scroll.0 + buffer_rect.height {
        editor.scroll.0 = (editor.cursor.1 as u16) - buffer_rect.height.saturating_sub(1);
    }
    
    f.set_cursor(editor.cursor.0 as u16, editor.cursor.1 as u16 - editor.scroll.0);
    
    
    let buffer_block = Paragraph::new(editor.lines.join("\n")).scroll(editor.scroll);
    
    let statusline_block = Paragraph::new(format!("[{}] {}:{}", editor.mode, editor.cursor.0, editor.cursor.1));
    let mut statusline_rect = f.size();
    statusline_rect.height = 1;
    statusline_rect.y = f.size().height - 2;
    
    f.render_widget(buffer_block, buffer_rect);
    f.render_widget(statusline_block, statusline_rect);
}
