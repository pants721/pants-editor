use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};

use crate::editor::{CurrentScreen, EditMode, Editor};

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
            Constraint::Min(line_number_width(editor) as u16), // line numbers
            Constraint::Max(1),                                // spacer
            Constraint::Min(f.size().width - (line_number_width(editor) as u16)), // editor
                                                               // content
        ])
        .split(full_layout[0]);

    // Main text
    f.render_widget(editor.widget(), buffer_layout[2]);

    // Line numbers
    if editor.settings.line_numbers {
        f.render_widget(line_numbers(editor), buffer_layout[0]);
    }

    // Cursor
    if editor.mode == EditMode::Insert || editor.mode == EditMode::Normal {
        let cursor_x = editor.cursor.x + buffer_layout[2].x as usize;
        let cursor_y = (editor.cursor.y + buffer_layout[2].y as usize - editor.scroll.0 as usize)
            .clamp(0, buffer_layout[2].height as usize - 1);
        f.set_cursor(cursor_x as u16, cursor_y as u16);
    } else if editor.mode == EditMode::Command || editor.mode == EditMode::Search {
        let cursor_x = (editor.command_x + full_layout[2].x as usize) + 1;
        let cursor_y = full_layout[2].y as usize;
        f.set_cursor(cursor_x as u16, cursor_y as u16);
    }

    // Status stuff
    f.render_widget(statusline(editor), full_layout[1]);
    f.render_widget(statusmessage(editor), full_layout[2]);

    // Exit popup
    if let CurrentScreen::Exiting = editor.current_screen {
        let area = centered_rect(60, 25, f.size());
        f.render_widget(Clear, area);
        f.render_widget(exit_popup(editor), area);
    }
}

fn line_numbers(editor: &Editor) -> Paragraph {
    let nums = (1..editor.lines.len() + 1).collect_vec();
    Paragraph::new(nums.into_iter().join("\n"))
        .style(editor.theme().primary_style())
        .dark_gray()
        .scroll(editor.scroll)
}

fn line_number_width(editor: &Editor) -> usize {
    if !editor.settings.line_numbers {
        return 0;
    }

    editor.lines.len().to_string().len() + 1
}

fn statusline(editor: &Editor) -> Paragraph {
    Paragraph::new(format!(
        "[{}] {}:{}",
        editor.mode,
        editor.cursor.x + 1,
        editor.cursor.y + 1
    ))
    .style(
        Style::default()
            .bg(editor.theme().statusline_bg)
            .fg(editor.theme().statusline_fg),
    )
}

fn statusmessage(editor: &Editor) -> Paragraph {
    if editor.mode == EditMode::Command {
        return Paragraph::new(":".to_string() + &editor.command).style(editor.theme().primary_style());
    } else if editor.mode == EditMode::Search {
        return Paragraph::new("/".to_string() + &editor.search.query).style(editor.theme().primary_style());
    } else {
        return Paragraph::new(editor.status_message.clone()).style(editor.theme().primary_style());
    }
}

fn exit_popup(editor: &Editor) -> Paragraph {
    let popup_block = Block::default()
        .style(editor.theme().primary_style())
        .borders(Borders::ALL);

    // the `trim: false` will stop the text from being cut off when over the edge of the block
    return Paragraph::new("Your changes are unsaved. Are you sure you would like to exit? (y/n)")
        .block(popup_block);
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
