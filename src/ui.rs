use itertools::Itertools;
use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::editor::{self, CurrentScreen, Editor};

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

    if let CurrentScreen::Exiting = editor.current_screen {
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area);
        
        let popup_block = Block::default()
            .borders(Borders::ALL);

        let exit_text = Text::styled(
            "Your changes are unsaved. Are you sure you would like to exit? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        f.render_widget(exit_paragraph, area);
    }
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
