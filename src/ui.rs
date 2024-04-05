use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::editor::Editor;

pub fn ui(f: &mut Frame, editor: &Editor) {
    f.set_cursor(editor.cursor.0 as u16, editor.cursor.1 as u16);
    let buffer_block = Paragraph::new(editor.lines.join("\n"));
    
    f.render_widget(buffer_block, f.size());
}
