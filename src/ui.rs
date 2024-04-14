use itertools::Itertools;
use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::editor::{self, Editor};

pub fn ui(f: &mut Frame, editor: &mut Editor) {
    let full_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(f.size().height - 2),
            Constraint::Max(1), // status line
            Constraint::Max(1), // notif area
        ])
        .split(f.size());
    
    let buffer_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Max(editor.lines.len().to_string().len() as u16), // line numbers
            Constraint::Max(1), // spacer
            Constraint::Max(f.size().width - (editor.lines.len().to_string().len() as u16)), // editor
            // content
        ])
        .split(full_layout[0]);
    
    let nums = (1..editor.lines.len()+1).collect_vec();
    let lnum_widget = Paragraph::new(nums.into_iter().join("\n")).dark_gray();

    f.render_widget(lnum_widget, buffer_layout[0]);
    f.render_widget(editor.widget(), buffer_layout[2]);

    let statusline_block = Paragraph::new(format!(
        "[{}] {}:{}",
        editor.mode, editor.cursor.x, editor.cursor.y
    ))
    .on_red()
    .black();
    f.render_widget(statusline_block, full_layout[1]);

    let statusmessage_block = Paragraph::new(editor.status_message.clone());
    f.render_widget(statusmessage_block, full_layout[2]);

    let cursor_x = editor.cursor.x + buffer_layout[2].x as usize;
    let cursor_y = (editor.cursor.y + buffer_layout[2].y as usize).clamp(0, full_layout[0].height as usize - 1);
    f.set_cursor(cursor_x as u16, cursor_y as u16);
}
