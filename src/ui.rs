use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph, Wrap}};

use crate::editor::{Editor, EditMode};

pub fn ui(f: &mut Frame, editor: &Editor) {
    match editor.mode {
        EditMode::Insert => f.set_cursor(editor.cursor.0 as u16, editor.cursor.1 as u16),
        EditMode::Normal => f.set_cursor(editor.cursor.0 as u16, editor.cursor.1 as u16),
    }
    let buffer_block = Paragraph::new(editor.lines.join("\n"));
    f.render_widget(buffer_block, f.size());
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
